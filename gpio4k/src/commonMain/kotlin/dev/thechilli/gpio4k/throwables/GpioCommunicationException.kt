package dev.thechilli.gpio4k.throwables

/**
 * Exception thrown when an error occurs during communication with a GPIO device.
 */
class GpioCommunicationException(
    message: String? = null,
    cause: Throwable? = null,
) : GpioException(message, cause)
