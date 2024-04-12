package dev.thechilli.gpio4k.pwm

/**
 * Generic PWM pin interface.
 */
interface PwmPin : AutoCloseable {
    val enabled: Boolean

    fun enable()
    fun disable()

    val periodNs: Long
    val dutyCycleNs: Long
    val ratio: Double
        get() = dutyCycleNs.toDouble() / periodNs
    val activeLow: Boolean

    fun setPeriodNs(periodNs: Long): PwmPin
    fun setDutyCycleNs(dutyCycleNs: Long): PwmPin
    fun setRatio(ratio: Double): PwmPin {
        require(ratio in 0.0..1.0) { "Ratio must be between 0.0 and 1.0" }
        return setDutyCycleNs((periodNs * ratio).toLong())
    }
    fun setActiveLow(activeLow: Boolean): PwmPin

    fun reset() {
        disable()
        setPeriodNs(1000000)
        setDutyCycleNs(0)
        setActiveLow(false)
    }
}
