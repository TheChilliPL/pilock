package dev.thechilli.gpio4k

import dev.thechilli.gpio4k.results.Result

interface DigitalInputOutput : DigitalInput, DigitalOutput {
    var ioMode: IOMode

    val isInput get() = ioMode == IOMode.INPUT
    val isOutput get() = ioMode == IOMode.OUTPUT

    val pullUpDownMode: PullUpDownMode

    /**
     * Set the pull up/down mode for the pin.
     *
     * @throws UnsupportedOperationException if the platform does not support pull up/down mode.
     */
    fun setPullUpDownMode(mode: PullUpDownMode)
    /**
     * Set the pull up/down mode for the pin.
     *
     * @return [Result.Success] if the mode was set successfully, [Result.Failure] if the platform does not support pull up/down mode.
     */
    fun trySetPullUpDownMode(mode: PullUpDownMode): Result<Unit, UnsupportedOperationException, Nothing> =
        Result.Failure.of(UnsupportedOperationException("Pull up/down mode is not supported"))
}
