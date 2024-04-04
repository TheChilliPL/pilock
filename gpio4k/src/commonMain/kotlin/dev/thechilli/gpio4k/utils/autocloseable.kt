package dev.thechilli.gpio4k.utils

fun <T : AutoCloseable> T.use(block: (T) -> Unit) {
    try {
        block(this)
    } finally {
        this.close()
    }
}
