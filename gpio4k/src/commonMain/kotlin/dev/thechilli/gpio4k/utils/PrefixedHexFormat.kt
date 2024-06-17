package dev.thechilli.gpio4k.utils

val prefixedHexFormat = HexFormat {
    upperCase = true
    number {
        prefix = "0x"
    }
}
