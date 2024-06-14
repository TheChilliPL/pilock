package dev.thechilli.gpio4k.utils

class ConioKeyReader : KeyReader {
    override fun initialize() {
        setInputEcho(false)
    }

    private var key: UByte? = null
    override fun update() {
        key = tryReadChar()
    }

    override fun readKey(): UByte? {
        return key
    }

    override fun close() {
        setInputEcho(true)
    }
}
