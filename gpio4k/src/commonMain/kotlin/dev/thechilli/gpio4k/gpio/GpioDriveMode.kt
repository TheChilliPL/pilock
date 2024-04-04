package dev.thechilli.gpio4k.gpio

enum class GpioDriveMode {
    /**
     * The output is driven high or low.
     */
    PUSH_PULL,

    /**
     * The output can be driven low, but is high impedance when high.
     */
    OPEN_DRAIN,

    /**
     * The output can be driven high, but is high impedance when low.
     */
    OPEN_SOURCE
}
