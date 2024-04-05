package dev.thechilli.gpio4k.gpio

/**
 * Generic GPIO pin interface.
 */
interface GpioPin : AutoCloseable {
    /**
     * Reads the current value of the pin.
     *
     * @throws GpioException if the pin is not readable
     */
    fun read(): Boolean

    /**
     * Writes a value to the pin.
     *
     * @throws GpioException if the pin is not writable
     */
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
