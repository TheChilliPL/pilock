package dev.thechilli.gpio4k.utils

interface KeyReader : AutoCloseable {
    fun initialize()

    fun update()
    fun readKey(): UByte?

    override fun close()
}
