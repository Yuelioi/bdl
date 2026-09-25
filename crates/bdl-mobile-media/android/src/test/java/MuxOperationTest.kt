package com.yueli.bdl.media

import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

class MuxOperationTest {
  @Test
  fun nativeLoadFailuresBecomeReportableExceptions() {
    for (failure in listOf(
      UnsatisfiedLinkError("missing FFmpeg JNI library"),
      NoClassDefFoundError("Could not initialize FFmpeg"),
      ExceptionInInitializerError(IllegalStateException("loader failed")),
    )) {
      try {
        muxWithNativeErrorHandling { throw failure }
        fail("Expected a reportable exception")
      } catch (error: Exception) {
        assertSame(failure, error.cause)
        assertTrue(error.message.orEmpty().contains("FFmpeg"))
      }
    }
  }

  @Test
  fun normalMuxErrorsKeepTheirDiagnostic() {
    val failure = IllegalStateException("disk full")
    try {
      muxWithNativeErrorHandling { throw failure }
      fail("Expected the mux error")
    } catch (error: Exception) {
      assertSame(failure, error)
    }
  }
}
