package com.yueli.bdl.media

import com.fasterxml.jackson.databind.ObjectMapper
import org.junit.Assert.assertEquals
import org.junit.Test

class MuxArgsTest {
  @Test
  fun deserializesTauriMuxPayload() {
    val args = ObjectMapper().readValue(
      """{"videoPath":"/tmp/video.m4s","audioPath":"/tmp/audio.m4s","outputPath":"/tmp/output.mkv","formatName":"matroska"}""",
      MuxArgs::class.java,
    )

    assertEquals("/tmp/video.m4s", args.videoPath)
    assertEquals("/tmp/audio.m4s", args.audioPath)
    assertEquals("/tmp/output.mkv", args.outputPath)
    assertEquals("matroska", args.formatName)
  }
}
