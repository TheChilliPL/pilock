package dev.thechilli.gpio4k.gpio

import kotlin.random.Random

class MockedGpioPin(
        val name: String,
) : GpioPin {
    var warnOnReadDangling = true

    /**
     * External state of the pin.
     * `true` means HIGH, `false` means LOW, `null` means high impedance.
     */
    var externalState: Boolean? = null

    /**
     * Internal state of the pin.
     * `true` means active, `false` means inactive, `null` means not outputting.
     * Both logical states can be inverted by `activeLow`.
     */
    var internallyExpected: Boolean? = null

    fun getInternalState(): Boolean? {
        var state = internallyExpected
        if(state != null && activeLow) state = !state
        if(state == true && drive == GpioDriveMode.OPEN_DRAIN)
            state = null
        else if(state == false && drive == GpioDriveMode.OPEN_SOURCE)
            state = null
        if(state == null) {
            if(bias == GpioLineBias.PULL_UP) state = true
            else if(bias == GpioLineBias.PULL_DOWN) state = false
        }
        return state
    }

    /**
     * @return `true` when HIGH, `false` when LOW, `null` when high impedance.
     * @throws GpioException if there is a short circuit.
     */
    fun getFinalState(): Boolean? {
        val internalState = getInternalState()

        if(internalState == null) return externalState

        if(internalState != externalState) {
            throw GpioException("Internal state of pin $name is ${if(internalState) "HIGH" else "LOW"}, but external state is ${if(externalState!!) "HIGH" else "LOW"}. This is a short circuit!")
        }

        return internalState
    }

    override fun read(): Boolean {
        if (mode == GpioIOMode.OUTPUT) {
            throw GpioException("Pin is not readable")
        }

        if (externalState == null) {
            if (warnOnReadDangling) {
                println("Warning: reading from pin $name with high impedance")
            }
            return Random.nextBoolean()
        }

        var state = externalState!!
        if (activeLow)
            state = !state
        return state
    }

    override fun write(value: Boolean) {
        if (mode == GpioIOMode.INPUT) {
            throw GpioException("Pin $name is not writable")
        }

        internallyExpected = value
    }

    override var mode: GpioIOMode = GpioIOMode.INPUT
        protected set

    override var activeLow: Boolean = false
        protected set

    override var bias: GpioLineBias = GpioLineBias.NONE
        protected set

    override var drive: GpioDriveMode = GpioDriveMode.PUSH_PULL
        protected set

    override fun setMode(mode: GpioIOMode): GpioPin {
        this.mode = mode
        if(mode == GpioIOMode.INPUT) {
            internallyExpected = null
        }
        return this
    }

    override fun setActiveLow(activeLow: Boolean): GpioPin {
        this.activeLow = activeLow
        return this
    }

    override fun setBias(bias: GpioLineBias): GpioPin {
        this.bias = bias
        return this
    }

    override fun setDrive(drive: GpioDriveMode): GpioPin {
        this.drive = drive
        return this
    }

    override fun close() {
        // Nothing to do
    }
}
