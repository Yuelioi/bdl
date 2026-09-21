package com.yueli.bdl.media;

import java.nio.charset.StandardCharsets;
import org.bytedeco.ffmpeg.avcodec.AVPacket;
import org.bytedeco.ffmpeg.avformat.AVFormatContext;
import org.bytedeco.ffmpeg.avformat.AVIOContext;
import org.bytedeco.ffmpeg.avformat.AVStream;
import org.bytedeco.ffmpeg.avutil.AVDictionary;
import org.bytedeco.ffmpeg.avutil.AVRational;
import org.bytedeco.javacpp.PointerPointer;

import static org.bytedeco.ffmpeg.global.avcodec.FF_COMPLIANCE_UNOFFICIAL;
import static org.bytedeco.ffmpeg.global.avcodec.av_packet_alloc;
import static org.bytedeco.ffmpeg.global.avcodec.av_packet_free;
import static org.bytedeco.ffmpeg.global.avcodec.av_packet_rescale_ts;
import static org.bytedeco.ffmpeg.global.avcodec.av_packet_unref;
import static org.bytedeco.ffmpeg.global.avcodec.avcodec_parameters_copy;
import static org.bytedeco.ffmpeg.global.avformat.AVFMT_NOFILE;
import static org.bytedeco.ffmpeg.global.avformat.AVIO_FLAG_WRITE;
import static org.bytedeco.ffmpeg.global.avformat.av_interleaved_write_frame;
import static org.bytedeco.ffmpeg.global.avformat.av_read_frame;
import static org.bytedeco.ffmpeg.global.avformat.av_write_trailer;
import static org.bytedeco.ffmpeg.global.avformat.avformat_alloc_output_context2;
import static org.bytedeco.ffmpeg.global.avformat.avformat_close_input;
import static org.bytedeco.ffmpeg.global.avformat.avformat_find_stream_info;
import static org.bytedeco.ffmpeg.global.avformat.avformat_free_context;
import static org.bytedeco.ffmpeg.global.avformat.avformat_new_stream;
import static org.bytedeco.ffmpeg.global.avformat.avformat_open_input;
import static org.bytedeco.ffmpeg.global.avformat.avformat_write_header;
import static org.bytedeco.ffmpeg.global.avformat.avio_closep;
import static org.bytedeco.ffmpeg.global.avformat.avio_open;
import static org.bytedeco.ffmpeg.global.avutil.AVMEDIA_TYPE_AUDIO;
import static org.bytedeco.ffmpeg.global.avutil.AVMEDIA_TYPE_VIDEO;
import static org.bytedeco.ffmpeg.global.avutil.AVERROR_EOF;
import static org.bytedeco.ffmpeg.global.avutil.AV_NOPTS_VALUE;
import static org.bytedeco.ffmpeg.global.avutil.av_compare_ts;
import static org.bytedeco.ffmpeg.global.avutil.av_strerror;

/**
 * Android stream-copy muxer backed by the FFmpeg libraries shipped in the APK.
 *
 * <p>The downloaded Bilibili DASH video/audio streams are already encoded, so this only remuxes
 * their first matching tracks into MP4 or Matroska. No decode or re-encode step is performed.</p>
 */
final class FfmpegMuxer {
  private FfmpegMuxer() {}

  static void mux(String videoPath, String audioPath, String outputPath, String formatName) {
    if (videoPath == null && audioPath == null) {
      throw new IllegalArgumentException("缺少视频或音频输入");
    }
    if (!"mp4".equals(formatName) && !"matroska".equals(formatName)) {
      throw new IllegalArgumentException("不支持的媒体封装格式：" + formatName);
    }

    InputTrack video = null;
    InputTrack audio = null;
    AVFormatContext output = new AVFormatContext(null);
    boolean outputIoOpen = false;
    boolean headerWritten = false;
    try {
      if (videoPath != null) {
        video = openInput(videoPath, AVMEDIA_TYPE_VIDEO, "视频");
      }
      if (audioPath != null) {
        audio = openInput(audioPath, AVMEDIA_TYPE_AUDIO, "音频");
      }

      check(
          avformat_alloc_output_context2(output, null, formatName, outputPath),
          "FFmpeg 无法创建输出上下文");
      if (output.isNull()) {
        throw new IllegalStateException("FFmpeg 未返回可用的输出上下文");
      }

      // Match desktop `ffmpeg -strict unofficial` so Dolby Vision side data is not rejected.
      output.strict_std_compliance(FF_COMPLIANCE_UNOFFICIAL);

      if (video != null) {
        addOutputStream(output, video, "视频");
      }
      if (audio != null) {
        addOutputStream(output, audio, "音频");
      }

      if ((output.oformat().flags() & AVFMT_NOFILE) == 0) {
        AVIOContext io = new AVIOContext(null);
        check(avio_open(io, outputPath, AVIO_FLAG_WRITE), "FFmpeg 无法打开输出文件");
        output.pb(io);
        outputIoOpen = true;
      }

      check(avformat_write_header(output, (AVDictionary) null), "FFmpeg 无法写入文件头");
      headerWritten = true;

      if (video != null) {
        video.loadNextPacket();
      }
      if (audio != null) {
        audio.loadNextPacket();
      }

      while ((video != null && video.hasPacket) || (audio != null && audio.hasPacket)) {
        final InputTrack next;
        if (video == null || !video.hasPacket) {
          next = audio;
        } else if (audio == null || !audio.hasPacket) {
          next = video;
        } else {
          next = comparePacketTime(video, audio) <= 0 ? video : audio;
        }

        writePacket(output, next);
        next.loadNextPacket();
      }

      check(av_write_trailer(output), "FFmpeg 无法完成媒体封装");
      headerWritten = false;
    } finally {
      if (headerWritten && output != null && !output.isNull()) {
        // Best effort only: the original exception remains the useful diagnostic.
        av_write_trailer(output);
      }
      if (outputIoOpen && output != null && !output.isNull() && output.pb() != null && !output.pb().isNull()) {
        avio_closep(output.pb());
      }
      if (output != null && !output.isNull()) {
        avformat_free_context(output);
      }
      closeInput(video);
      closeInput(audio);
    }
  }

  private static InputTrack openInput(String path, int mediaType, String label) {
    AVFormatContext context = new AVFormatContext(null);
    try {
      check(avformat_open_input(context, path, null, (AVDictionary) null), "FFmpeg 无法打开" + label + "输入");
      check(
          avformat_find_stream_info(context, (PointerPointer) null),
          "FFmpeg 无法读取" + label + "流信息");

      int streamIndex = -1;
      AVStream stream = null;
      for (int index = 0; index < context.nb_streams(); index++) {
        AVStream candidate = context.streams(index);
        if (candidate != null
            && !candidate.isNull()
            && candidate.codecpar() != null
            && candidate.codecpar().codec_type() == mediaType) {
          streamIndex = index;
          stream = candidate;
          break;
        }
      }
      if (streamIndex < 0 || stream == null) {
        throw new IllegalStateException(label + "输入中没有可用" + label + "轨道");
      }

      AVPacket packet = av_packet_alloc();
      if (packet == null || packet.isNull()) {
        throw new IllegalStateException("FFmpeg 无法分配媒体数据包");
      }
      return new InputTrack(context, stream, streamIndex, packet);
    } catch (RuntimeException error) {
      if (context != null && !context.isNull()) {
        avformat_close_input(context);
      }
      throw error;
    }
  }

  private static void addOutputStream(AVFormatContext output, InputTrack input, String label) {
    AVStream stream = avformat_new_stream(output, null);
    if (stream == null || stream.isNull()) {
      throw new IllegalStateException("FFmpeg 无法创建" + label + "输出轨道");
    }
    check(
        avcodec_parameters_copy(stream.codecpar(), input.inputStream.codecpar()),
        "FFmpeg 无法复制" + label + "编码参数");
    stream.codecpar().codec_tag(0);
    stream.time_base(input.inputStream.time_base());
    input.outputStream = stream;
  }

  private static int comparePacketTime(InputTrack left, InputTrack right) {
    long leftTs = packetTimestamp(left.packet);
    long rightTs = packetTimestamp(right.packet);
    if (leftTs == AV_NOPTS_VALUE && rightTs == AV_NOPTS_VALUE) {
      return 0;
    }
    if (leftTs == AV_NOPTS_VALUE) {
      return 1;
    }
    if (rightTs == AV_NOPTS_VALUE) {
      return -1;
    }
    return av_compare_ts(leftTs, left.inputStream.time_base(), rightTs, right.inputStream.time_base());
  }

  private static long packetTimestamp(AVPacket packet) {
    return packet.dts() != AV_NOPTS_VALUE ? packet.dts() : packet.pts();
  }

  private static void writePacket(AVFormatContext output, InputTrack input) {
    AVPacket packet = input.packet;
    av_packet_rescale_ts(packet, input.inputStream.time_base(), input.outputStream.time_base());
    packet.stream_index(input.outputStream.index());
    packet.pos(-1);
    int result = av_interleaved_write_frame(output, packet);
    av_packet_unref(packet);
    check(result, "FFmpeg 写入媒体数据失败");
    input.hasPacket = false;
  }

  private static void closeInput(InputTrack input) {
    if (input == null) {
      return;
    }
    if (input.packet != null && !input.packet.isNull()) {
      av_packet_free(input.packet);
    }
    if (input.context != null && !input.context.isNull()) {
      avformat_close_input(input.context);
    }
  }

  private static void check(int result, String operation) {
    if (result < 0) {
      throw new IllegalStateException(operation + "：" + ffmpegError(result) + " (" + result + ")");
    }
  }

  private static String ffmpegError(int errorCode) {
    byte[] buffer = new byte[256];
    if (av_strerror(errorCode, buffer, buffer.length) < 0) {
      return "未知 FFmpeg 错误";
    }
    int length = 0;
    while (length < buffer.length && buffer[length] != 0) {
      length++;
    }
    return new String(buffer, 0, length, StandardCharsets.UTF_8);
  }

  private static final class InputTrack {
    final AVFormatContext context;
    final AVStream inputStream;
    final int inputStreamIndex;
    final AVPacket packet;
    AVStream outputStream;
    boolean hasPacket;
    boolean eof;

    InputTrack(AVFormatContext context, AVStream inputStream, int inputStreamIndex, AVPacket packet) {
      this.context = context;
      this.inputStream = inputStream;
      this.inputStreamIndex = inputStreamIndex;
      this.packet = packet;
    }

    void loadNextPacket() {
      hasPacket = false;
      if (eof) {
        return;
      }

      while (true) {
        int result = av_read_frame(context, packet);
        if (result < 0) {
          eof = true;
          av_packet_unref(packet);
          if (result != AVERROR_EOF) {
            check(result, "FFmpeg 读取媒体数据失败");
          }
          return;
        }
        if (packet.stream_index() == inputStreamIndex) {
          hasPacket = true;
          return;
        }
        av_packet_unref(packet);
      }
    }
  }
}
