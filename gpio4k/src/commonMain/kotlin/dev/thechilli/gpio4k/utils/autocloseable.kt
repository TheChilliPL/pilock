package dev.thechilli.gpio4k.utils

/**
 * Allows you to use an [AutoCloseable] object in the [block] and automatically close it afterward.
 */
fun <T : AutoCloseable> T.use(block: (T) -> Unit) {
    try {
        block(this)
    } finally {
        this.close()
    }
}

/**
 * Allows you to use a collection of [AutoCloseable] objects in the [block] and automatically close them afterward.
 */
fun <T : Collection<AutoCloseable>> T.useAll(block: (T) -> Unit) {
    try {
        block(this)
    } finally {
        this.forEach { it.close() }
    }
}

class ClosingScope {
    private val closeables = mutableListOf<AutoCloseable>()

    fun <T : AutoCloseable> T.autoClose(): T = apply {
        closeables.add(this)
    }

    fun <T : Collection<AutoCloseable>> T.autoCloseAll(): T = apply {
        closeables.addAll(this)
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

/**
 * Creates a new [ClosingScope] and executes the given [block] on it.
 *
 * The [ClosingScope] will automatically close all [AutoCloseable] objects added to it by [ClosingScope.autoClose]
 * and [ClosingScope.autoCloseAll].
 * These methods return the [AutoCloseable] object they were called on, so they can easily be used in variable declarations.
 */
fun closingScope(block: ClosingScope.() -> Unit) {
    ClosingScope.use(block)
}
