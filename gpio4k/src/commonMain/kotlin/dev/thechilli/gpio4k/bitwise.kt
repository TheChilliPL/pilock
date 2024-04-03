package dev.thechilli.gpio4k

fun UByte.bitFromLeft(bitIndex: Int): Boolean = (this.toInt() and (1 shl bitIndex)) != 0

fun UByte.bitFromRight(bitIndex: Int): Boolean = (this.toInt() and (1 shl (7 - bitIndex))) != 0
