package dev.thechilli.gpio4k.utils

fun <T : AutoCloseable> T.use(block: (T) -> Unit) {
    try {
        block(this)
    } finally {
        this.close()
    }
}

fun <T : Collection<AutoCloseable>> T.useAll(block: (T) -> Unit) {
    try {
        block(this)
    } finally {
        this.forEach { it.close() }
    }
}

class ClosingScope {
    private val closeables = mutableListOf<AutoCloseable>()

    operator fun AutoCloseable.unaryPlus() {
        closeables.add(this)
    }

    private fun close() {
        val exceptions = mutableListOf<Throwable>()
        closeables.forEach {
            try {
                it.close()
            } catch (e: Throwable) {
                exceptions.add(e)
            }
        }
        if (exceptions.isNotEmpty()) {
            val exception = exceptions.removeAt(0)
            exceptions.forEach { exception.addSuppressed(it) }
            throw exception
        }
    }

    companion object {
        fun use(block: ClosingScope.() -> Unit) {
            val scope = ClosingScope()
            try {
                scope.block()
            } finally {
                scope.close()
            }
        }
    }
}
