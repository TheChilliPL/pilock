package dev.thechilli.gpio4k

interface DigitalOutput : AutoCloseable {
    fun write(state: DigitalState)

    /**
     * Close should free the pins or any other resources used by the input.
     */
    override fun close()
}
