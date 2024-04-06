package dev.thechilli.gpio4k.lcd

import dev.thechilli.gpio4k.utils.bitFromLeft

interface HD44780Display : CharacterDisplay {
    val getLineOffsets: List<UByte>

    val readingAvailable: Boolean
    val currentAddress: UByte
    val currentlyInCgRam: Boolean

    fun lineOfAddress(address: UByte): Int {
        return getLineOffsets
            .filter { it <= address }
            .withIndex()
            .minBy { address - it.value }.index
    }

    val characterRom: HD44780CharacterSet

    override fun writeChar(char: Char) {
        writeData(true, characterRom.codeOf(char))
    }

    override fun breakLine() {
        // TODO Make sure this works with screen scrolling
        val currentLine = lineOfAddress(currentAddress)
        val nextLine = (currentLine + 1).mod(getLineOffsets.size)
        setDdRamAddress(getLineOffsets[nextLine])
    }

    fun readChar(): Char {
        return characterRom[readData(true)]
    }

    override fun clearDisplay() {
        writeData(false, 0x01u)
    }

    override fun returnHome() {
        writeData(false, 0x02u)
    }

    fun entryModeSet(
        increment: Boolean,
        shift: Boolean
    ) {
        val data = 0x04u or (if (increment) 0x02u else 0u) or (if (shift) 0x01u else 0u)
        writeData(false, data.toUByte())
    }

    fun displayControl(
        displayOn: Boolean = true,
        cursorOn: Boolean = false,
        cursorBlink: Boolean = false
    ) {
        val data =
            0x08u or (if (displayOn) 0x04u else 0u) or (if (cursorOn) 0x02u else 0u) or (if (cursorBlink) 0x01u else 0u)
        writeData(false, data.toUByte())
    }

    fun cursorDisplayShift(
        displayShift: Boolean,
        right: Boolean
    ) {
        val data = 0x10u or (if (displayShift) 0x08u else 0u) or (if (right) 0x04u else 0u)
        writeData(false, data.toUByte())
    }

    override fun shiftCursor(direction: CursorDirection) {
        cursorDisplayShift(false, direction == CursorDirection.Right)
    }

    override fun shiftDisplay(direction: CursorDirection) {
        cursorDisplayShift(true, direction == CursorDirection.Right)
    }

    fun functionSet(
        dataLength8Bit: Boolean = true,
        twoLines: Boolean = true,
        font5x10: Boolean = false
    ) {
        val data =
            0x20u or (if (dataLength8Bit) 0x10u else 0u) or (if (twoLines) 0x08u else 0u) or (if (font5x10) 0x04u else 0u)
        writeData(false, data.toUByte())
    }

    fun setCgRamAddress(address: UByte) {
        val data = 0x40u or (address and 0x3Fu).toUInt()
        writeData(false, data.toUByte())
    }

    fun setDdRamAddress(address: UByte) {
        val data = 0x80u or (address and 0x7Fu).toUInt()
        writeData(false, data.toUByte())
    }

    override fun setCursor(row: Int, column: Int) {
        setDdRamAddress((getLineOffsets[row] + column.toUByte()).toUByte())
    }

    fun readBusyAndAddress(): UByte {
        return readData(false)
    }

    override fun readBusyFlag(): Boolean {
        return readBusyAndAddress().bitFromLeft(0)
    }

    override fun readAddress(): UByte {
        return readBusyAndAddress() and 0b0111_1111u
    }

    fun writeData(rs: Boolean, data: UByte)
    fun readData(rs: Boolean): UByte

    companion object {
        private const val NUL = '\u0000'

        // Character ROM A00
        // https://www.sparkfun.com/datasheets/LCD/HD44780.pdf#page=17
        val ROM_A00 = HD44780CharacterSet.of(
            NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL,
            NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL,
            ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
            '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
            'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '¥', ']', '^', '_',
            '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
            'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '←', '→',
            NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL,
            '\u3000', '。', '「', '」', '、', '・', 'ヲ', 'ァ', 'ィ', 'ゥ', 'ェ', 'ォ', 'ャ', 'ュ', 'ョ', 'ッ',
            'ー', 'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス', 'セ', 'ソ',
            'タ', 'チ', 'ツ', 'テ', 'ト', 'ナ', 'ニ', 'ヌ', 'ネ', 'ノ', 'ハ', 'ヒ', 'フ', 'ヘ', 'ホ', 'マ',
            'ミ', 'ム', 'メ', 'モ', 'ヤ', 'ユ', 'ヨ', 'ラ', 'リ', 'ル', 'レ', 'ロ', 'ワ', 'ン', '゛', '゜',
            // TODO Fill missing characters
            'α', 'ä', 'β', 'ε', 'μ', 'σ', 'ρ', '√', NUL, NUL, NUL, NUL, NUL, NUL, 'ñ', 'ö',
            NUL, NUL, 'θ', '∞', 'Ω', 'ü', '∑', 'π', NUL, NUL, '千', '万', '円', '÷', NUL, '█',
        )

        // Character ROM A02
        // https://www.sparkfun.com/datasheets/LCD/HD44780.pdf#page=18
        val ROM_A02 = HD44780CharacterSet.of(
            NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL,
            '⯈', '⯇', '“', '”', '⏫', '⏬', '•', '↵', '↑', '↓', '→', '←', '≤', '≥', '⯅', '⯆',
            ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
            '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
            'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '¥', ']', '^', '_',
            '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
            'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~', '⌂',
            'Б', 'Д', 'Ж', 'З', 'И', 'Й', 'Л', 'П', 'У', 'Ц', 'Ч', 'Ш', 'Щ', 'Ъ', 'Ы', 'Э',
            'α', '♪', 'Г', 'π', 'Σ', 'σ', '♫', 'τ', '⍾', 'θ', 'Ω', 'δ', '∞', '❤', 'ε', '⋂',
            '⏸', 'ℹ', '¢', '£', '¤', '¥', '¦', '§', '⨍', '©', 'ª', '≪', 'Ю', 'Я', '®', '‘',
            '°', '±', '²', '³', '₧', 'µ', '¶', '·', 'ω', '¹', 'º', '≫', '¼', '½', '¾', '¿',
            'À', 'Á', 'Â', 'Ã', 'Ä', 'Å', 'Æ', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ì', 'Í', 'Î', 'Ï',
            'Ð', 'Ñ', 'Ò', 'Ó', 'Ô', 'Õ', 'Ö', '×', 'Φ', 'Ù', 'Ú', 'Û', 'Ü', 'Ý', 'Þ', 'ß',
            'à', 'á', 'â', 'ã', 'ä', 'å', 'æ', 'ç', 'è', 'é', 'ê', 'ë', 'ì', 'í', 'î', 'ï',
            'ð', 'ñ', 'ò', 'ó', 'ô', 'õ', 'ö', '÷', 'φ', 'ù', 'ú', 'û', 'ü', 'ý', 'þ', 'ÿ', //ϕ?
        )
    }
}
