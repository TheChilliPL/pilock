package dev.thechilli.gpio4k.gpio

interface GpioPin : AutoCloseable {
    fun read(): Boolean
    fun write(value: Boolean)

    val mode: GpioIOMode
    val activeLow: Boolean
    val bias: GpioLineBias
    val drive: GpioDriveMode

    fun setMode(mode: GpioIOMode): GpioPin
    fun setActiveLow(activeLow: Boolean): GpioPin
    fun setBias(bias: GpioLineBias): GpioPin
    fun setDrive(drive: GpioDriveMode): GpioPin

    /**
     * Resets the pin to its default state.
     */
    fun reset(mode: GpioIOMode = GpioIOMode.INPUT) {
        setMode(mode)
        setActiveLow(false)
        setBias(GpioLineBias.NONE)
        setDrive(GpioDriveMode.PUSH_PULL)
    }
}
