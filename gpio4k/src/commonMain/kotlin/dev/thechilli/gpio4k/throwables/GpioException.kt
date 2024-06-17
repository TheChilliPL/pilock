package dev.thechilli.gpio4k.throwables

/**
 * Parent class for all GPIO-related exceptions.
 */
open class GpioException(
    message: String? = null,
    cause: Throwable? = null,
) : RuntimeException(message, cause)
