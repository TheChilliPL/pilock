package dev.thechilli.gpio4k.pwm

/**
 * Generic PWM pin interface.
 */
interface PwmPin : AutoCloseable {
    val enabled: Boolean

    fun enable()
    fun disable()

    /**
     * The period of the PWM signal in nanoseconds.
     * Period is the time it takes for the signal to repeat.
     */
    val periodNs: Long
    /**
     * The duty cycle of the PWM signal in nanoseconds.
     * Duty cycle is the time the signal is high in a period.
     */
    val dutyCycleNs: Long
    /**
     * The ratio of the duty cycle to the period.
     */
    val ratio: Double
        get() = dutyCycleNs.toDouble() / periodNs
    val activeLow: Boolean

    /**
     * Sets the period of the PWM signal in nanoseconds.
     * Period is the time it takes for the signal to repeat.
     */
    fun setPeriodNs(periodNs: Long): PwmPin
    /**
     * Sets the duty cycle of the PWM signal in nanoseconds.
     * Duty cycle is the time the signal is high in a period.
     */
    fun setDutyCycleNs(dutyCycleNs: Long): PwmPin
    /**
     * Sets the duty cycle so that the ratio of the duty cycle to the period is equal to the given [ratio].
     */
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
