package dev.thechilli.gpio4k.gpio

import dev.thechilli.gpio4k.throwables.GpioException
import dev.thechilli.gpio4k.utils.isDebug

/**
 * A GPIO pin that is controlled by the gpiod command line interface.
 *
 * gpiod uses ioctl calls with some very elaborate structures to control GPIO pins.
 */
class GpiodPin(val gpioChipId: Int, val pinId: Int) : GpioPin {
    override fun read(): Boolean {
        if(mode != GpioIOMode.INPUT)
            throw GpioException("Pin $pinId is not readable")

        // gpioget [-l] -B <bias> <chip> <pin>
        val args = mutableListOf<String>()
        // Set pin to active low if necessary
        if(activeLow) args.add("-l")
        // Set bias
        args.add("-B")
        args.add(when(bias) {
            GpioLineBias.NONE -> "disable"
            GpioLineBias.PULL_UP -> "pull-up"
            GpioLineBias.PULL_DOWN -> "pull-down"
        })
        // Pass chip and pin id
        args.add(gpioChipId.toString())
        args.add(pinId.toString())
        // Read the pin
        val (exitCode, output) = exec("gpiod", *args.toTypedArray())
        if(exitCode != 0)
            throw GpioException("Failed to read pin $pinId.\ngpioget exited with $exitCode.\n$output")
        return output.trim() == "1"
    }

    private var lastSetPid: Long = 0
    private var forceSet = true
    private var lastState = false

    override fun write(value: Boolean) {
        if(!forceSet && lastState == value) return // Skip if the value is the same

        if(mode != GpioIOMode.OUTPUT)
            throw GpioException("Pin $pinId is not writable")

        // Kill the last set command if it's still running
        if(lastSetPid != 0L)
            kill(lastSetPid)

        // gpioset [-l] -B <bias> -D <drive> -m signal <chip> <pin>=<value>
        val args = mutableListOf<String>()
        // Set pin to active low if necessary
        if(activeLow) args.add("-l")
        // Set bias
        args.add("-B")
        args.add(when(bias) {
            GpioLineBias.NONE -> "disable"
            GpioLineBias.PULL_UP -> "pull-up"
            GpioLineBias.PULL_DOWN -> "pull-down"
        })
        // Set drive mode
        args.add("-D")
        args.add(when(drive) {
            GpioDriveMode.PUSH_PULL -> "push-pull"
            GpioDriveMode.OPEN_DRAIN -> "open-drain"
            GpioDriveMode.OPEN_SOURCE -> "open-source"
        })
        // Keep pin state until SIGTERM
        args.add("-m")
        args.add("signal")
        // Pass chip and pin id
        args.add(gpioChipId.toString())
        args.add(pinId.toString() + "=" + if(value) "1" else "0")
        // Write the pin
        lastSetPid = spawn("gpioset", *args.toTypedArray())

        lastState = value
        forceSet = false
    }

    override var mode = GpioIOMode.INPUT
        private set

    override fun setMode(mode: GpioIOMode): GpioPin {
        if(this.mode == GpioIOMode.OUTPUT && mode == GpioIOMode.INPUT) {
            // Kill the last set command if it's still running
            if(lastSetPid != 0L) {
                kill(lastSetPid)
                lastSetPid = 0
            }
        }
        this.mode = mode
        forceSet = true
        return this
    }

    override var activeLow = false
        private set

    override fun setActiveLow(activeLow: Boolean): GpioPin {
        this.activeLow = activeLow
        forceSet = true
        return this
    }

    override var bias = GpioLineBias.NONE
        private set

    override fun setBias(bias: GpioLineBias): GpioPin {
        this.bias = bias
        forceSet = true
        return this
    }

    override var drive = GpioDriveMode.PUSH_PULL
        private set

    override fun setDrive(drive: GpioDriveMode): GpioPin {
        this.drive = drive
        forceSet = true
        return this
    }

    override fun close() {
        if(lastSetPid != 0L) {
            // Kill the last set command if it's still running
            kill(lastSetPid)
            lastSetPid = 0
        }
    }

    protected fun finalize() {
        if(isDebug) {
            if(lastSetPid != 0L) {
                // TODO Log this to error stream instead of stdout
                println("[DEBUG] GpiodPin $pinId has not been closed properly before destruction. Closing now.")
                kill(lastSetPid)
            }
        }
    }
}
