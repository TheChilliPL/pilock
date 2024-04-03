package dev.thechilli.gpio4k.results

import kotlin.test.Test
import kotlin.test.assertEquals

class ResultTest {
    @Test
    fun `Result should be a success`() {
        val result = Result.Success.of(1)

        assertEquals(1, result.getOrNull())
        assertEquals(null, result.getFailureOrNull())
        assertEquals(true, result.isSuccess)
    }

    @Test
    fun `Result should be a failure`() {
        val result = Result.Failure.of("error")

        assertEquals(null, result.getOrNull())
        assertEquals("error", result.getFailureOrNull())
        assertEquals(false, result.isSuccess)
    }

    @Test
    fun `Result should map success`() {
        val result = Result.Success.of(1)

        val mapped = result.map { it + 1 }

        assertEquals(2, mapped.getOrNull())
    }
}
