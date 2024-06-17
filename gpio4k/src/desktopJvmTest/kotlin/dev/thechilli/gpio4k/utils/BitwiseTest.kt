package dev.thechilli.gpio4k.utils

import kotlin.test.Test
import kotlin.test.assertEquals

class BitwiseTest {
    @Test
    fun `Test bit from left`() {
        val byte: UByte = 0b10110111u

        assertEquals(true, byte.bitFromLeft(0))
        assertEquals(false, byte.bitFromLeft(1))
        assertEquals(true, byte.bitFromLeft(2))
        assertEquals(true, byte.bitFromLeft(3))
        assertEquals(false, byte.bitFromLeft(4))
        assertEquals(true, byte.bitFromLeft(5))
        assertEquals(true, byte.bitFromLeft(6))
        assertEquals(true, byte.bitFromLeft(7))
    }

    @Test
    fun `Test bit from right`() {
        val byte: UByte = 0b10110111u

        assertEquals(true, byte.bitFromRight(0))
        assertEquals(true, byte.bitFromRight(1))
        assertEquals(true, byte.bitFromRight(2))
        assertEquals(false, byte.bitFromRight(3))
        assertEquals(true, byte.bitFromRight(4))
        assertEquals(true, byte.bitFromRight(5))
        assertEquals(false, byte.bitFromRight(6))
        assertEquals(true, byte.bitFromRight(7))
    }
}
