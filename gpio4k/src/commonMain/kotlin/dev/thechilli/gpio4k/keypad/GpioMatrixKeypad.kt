package dev.thechilli.gpio4k.keypad

import dev.thechilli.gpio4k.gpio.GpioIOMode
import dev.thechilli.gpio4k.gpio.GpioPin
import dev.thechilli.gpio4k.gpio.keepHigh
import dev.thechilli.gpio4k.gpio.resetAll

class GpioMatrixKeypad(
    private val keys : List<List<Char>>,
    private val rowPins : List<GpioPin>,
    private val columnPins : List<GpioPin>,
) : Keypad {
    init {
        require(keys.isNotEmpty()) { "Keys must not be empty" }
        require(rowPins.isNotEmpty()) { "Row pins must not be empty" }
        require(columnPins.isNotEmpty()) { "Column pins must not be empty" }

        require(keys.size == rowPins.size) { "Number of rows must match number of row pins" }
        require(keys[0].size == columnPins.size) { "Number of columns must match number of column pins" }
    }

    override fun initialize() {
        rowPins.resetAll(GpioIOMode.OUTPUT)
        columnPins.resetAll(GpioIOMode.INPUT)
    }

    override val rows: Int = keys.size
    override val columns: Int = keys[0].size

    override fun getKey(column: Int, row: Int): Char = keys[row][column]

    override fun readKeys(): List<Char> {
        val keys = mutableListOf<Char>()

        for (i in 0 until columns) {
            columnPins[i].keepHigh {
                for (j in 0 until rows) {
                    if (rowPins[j].read()) {
                        keys.add(getKey(i, j))
                    }
                }
            }
        }

        return keys
    }
}
