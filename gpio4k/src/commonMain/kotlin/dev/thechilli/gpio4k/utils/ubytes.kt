package dev.thechilli.gpio4k.utils

fun String.encodeToUByteArray(): UByteArray = this.encodeToByteArray().toUByteArray()

fun UByteArray.decodeToString(): String = this.toByteArray().decodeToString()
