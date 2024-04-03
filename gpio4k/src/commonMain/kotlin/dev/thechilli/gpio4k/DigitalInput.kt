package dev.thechilli.gpio4k

interface DigitalInput : AutoCloseable {
    fun read(): DigitalState

    fun isHigh(): Boolean
    fun isLow(): Boolean = !isHigh()

    /**
     * Close should free the pins or any other resources used by the input.
     */
    override fun close()
}
