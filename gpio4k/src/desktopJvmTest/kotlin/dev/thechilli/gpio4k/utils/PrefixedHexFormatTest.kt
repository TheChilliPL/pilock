package dev.thechilli.gpio4k.utils

import kotlin.test.Test
import kotlin.test.assertEquals

class PrefixedHexFormatTest {
    @Test
    fun `Converts byte`() {
        val byte = 0xA.toByte()
        val formatted = byte.toHexString(prefixedHexFormat)
        assertEquals("0x0A", formatted)
    }

    @Test
    fun `Converts short`() {
        val short = 0xA.toShort()
        val formatted = short.toHexString(prefixedHexFormat)
        assertEquals("0x000A", formatted)
    }

    @Test
    fun `Converts int`() {
        val int = 0xA
        val formatted = int.toHexString(prefixedHexFormat)
        assertEquals("0x0000000A", formatted)
    }

    @Test
    fun `Converts long`() {
        val long = 0xA.toLong()
        val formatted = long.toHexString(prefixedHexFormat)
        assertEquals("0x000000000000000A", formatted)
    }

    @Test
    fun `Converts unsigned byte`() {
        val byte = 0xA.toUByte()
        val formatted = byte.toHexString(prefixedHexFormat)
        assertEquals("0x0A", formatted)
    }

    @Test
    fun `Converts unsigned short`() {
        val short = 0xA.toUShort()
        val formatted = short.toHexString(prefixedHexFormat)
        assertEquals("0x000A", formatted)
    }

    @Test
    fun `Converts unsigned int`() {
        val int = 0xA.toUInt()
        val formatted = int.toHexString(prefixedHexFormat)
        assertEquals("0x0000000A", formatted)
    }

    @Test
    fun `Converts unsigned long`() {
        val long = 0xA.toULong()
        val formatted = long.toHexString(prefixedHexFormat)
        assertEquals("0x000000000000000A", formatted)
    }
}
