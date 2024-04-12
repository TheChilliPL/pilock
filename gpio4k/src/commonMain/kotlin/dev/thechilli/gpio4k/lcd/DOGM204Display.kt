package dev.thechilli.gpio4k.lcd

interface DOGM204Display : HD44780Display {
    override fun clearDisplay() {
        writeData(false, 0x01u, null, null)
    }

    override fun returnHome() {
        writeData(false, 0x02u, false, null)
    }

    fun powerDownMode(powerDown: Boolean) {
        writeData(false, if (powerDown) 0x03u else 0x02u, true, null)
    }

    override fun entryModeSet(increment: Boolean, shift: Boolean) {
        val data = 0x04u or (if (increment) 0x02u else 0u) or (if (shift) 0x01u else 0u)
        writeData(false, data.toUByte(), false, null)
    }

    fun dataShiftDirection(reverseCommon: Boolean, reverseSegment: Boolean) {
        val data = 0x04u or (if (reverseCommon) 0x02u else 0u) or (if (reverseSegment) 0x01u else 0u)
        writeData(false, data.toUByte(), true, null)
    }

    override fun displayControl(displayOn: Boolean, cursorOn: Boolean, cursorBlink: Boolean) {
        val data = 0x08u or (if (displayOn) 0x04u else 0u) or (if (cursorOn) 0x02u else 0u) or (if (cursorBlink)
            0x01u else 0u)
        writeData(false, data.toUByte(), false, null)
    }

    /**
     * @param nw True if 3 or 4 lines are used, false if 1 or 2 lines are used.
     */
    fun extendedFunctionSet(widerFont: Boolean, cursorInvert: Boolean, nw: Boolean) {
        val data = 0x08u or (if (widerFont) 0x04u else 0u) or (if (cursorInvert) 0x02u else 0u) or (if (nw) 0x01u
        else 0u)
        writeData(false, data.toUByte(), true, null)
    }

    override fun cursorDisplayShift(displayShift: Boolean, right: Boolean) {
        val data = 0x10u or (if (displayShift) 0x08u else 0u) or (if (right) 0x04u else 0u)
        writeData(false, data.toUByte(), false, false)
    }

    enum class DOGM204DoubleHeightConfiguration(val ud2: Boolean, val ud1: Boolean) {
        SINGLE_SINGLE_DOUBLE(false, false),
        SINGLE_DOUBLE_SINGLE(false, true),
        DOUBLE_DOUBLE(true, false),
        DOUBLE_SINGLE_SINGLE(true, true)
    }

    enum class DOGM204Bias(val bs1: Boolean, val bs0: Boolean) {
        BIAS_1_5(false, false),
        BIAS_1_4(false, true),
        BIAS_1_7(true, false),
        BIAS_1_3(true, true)
    }

    fun doubleHeightBiasShift(
        doubleHeightConfiguration: DOGM204DoubleHeightConfiguration,
        bs1: Boolean,
        displayShiftPerLine: Boolean,
    ) {
        val data = 0x10u or
                (if (doubleHeightConfiguration.ud2) 0x08u else 0u) or
                (if (doubleHeightConfiguration.ud1) 0x04u else 0u) or
                (if (bs1) 0x02u else 0u) or
                (if (displayShiftPerLine) 0x01u else 0u)
        writeData(false, data.toUByte(), true, false)
    }

    private fun getFrequencyId(frequencyHz: Int): UByte = when(frequencyHz) {
        680 -> 0b111u
        640 -> 0b110u
        620 -> 0b101u
        580 -> 0b100u
        540 -> 0b011u
        500 -> 0b010u
        460 -> 0b001u
        420 -> 0b000u
        else -> throw IllegalArgumentException("Unsupported frequency: $frequencyHz")
    }

    /**
     * @param frequencyHz One of the supported frequencies: 420, 460, 500, 540 (default), 580, 620, 640, 680.
     */
    fun configureOscillatorFrequency(bs0: Boolean, frequencyHz: Int) {
        val data = 0x10u or
                (if (bs0) 0x08u else 0u) or
                getFrequencyId(frequencyHz).toUInt()
        writeData(false, data.toUByte(), false, true)
    }

    fun shiftScrollEnable(line1: Boolean, line2: Boolean, line3: Boolean, line4: Boolean) {
        val data = 0x10u or
                (if (line1) 0x08u else 0u) or
                (if (line2) 0x04u else 0u) or
                (if (line3) 0x02u else 0u) or
                (if (line4) 0x01u else 0u)
        writeData(false, data.toUByte(), true, true)
    }

    override fun functionSet(dataLength8Bit: Boolean, twoLines: Boolean, font5x10: Boolean) {
        functionSetIs(dataLength8Bit, twoLines, font5x10, false)
    }

    fun functionSetIs(dataLength8Bit: Boolean, twoLines: Boolean, font5x10: Boolean, specialRegisters:
    Boolean) {
        val data = 0x20u or (if (dataLength8Bit) 0x10u else 0u) or (if (twoLines) 0x08u else 0u) or (if (font5x10)
            0x04u else 0u) or (if(specialRegisters) 0x01u else 0u)
        writeData(false, data.toUByte(), null, null)
    }

    fun functionSetRev(dataLength8Bit: Boolean, twoLines: Boolean, font5x10: Boolean, reverseDisplay:
    Boolean) {
        val data = 0x20u or (if (dataLength8Bit) 0x10u else 0u) or (if (twoLines) 0x08u else 0u) or (if (font5x10)
            0x04u else 0u) or 0x02u or (if (reverseDisplay) 0x01u else 0u)
        writeData(false, data.toUByte(), null, null)
    }

    override fun setCgRamAddress(address: UByte) {
        val data = 0x40u or (address and 0x3Fu).toUInt()
        writeData(false, data.toUByte(), false, false)
    }

    fun setSegRamAddress(address: UByte) {
        val data = 0x40u or (address and 0x0Fu).toUInt()
        writeData(false, data.toUByte(), false, true)
    }

    fun iconContrastControl(icon: Boolean, regulator: Boolean, contrast: UByte) {
        val data = 0x50u or (if (icon) 0x08u else 0u) or (if (regulator) 0x04u else 0u) or (contrast.toUInt() shr 3 and 0x03u)
        writeData(false, data.toUByte(), false, true)
    }

    fun followerControl(divider: Boolean, internalResistorRatio: Int) {
        require(internalResistorRatio in 0..7) { "Internal resistor ratio setting must be between 0 and 7, " +
                "corresponding to IR0–IR7 from the documentation." }

        val data = 0x60u or (if (divider) 0x08u else 0u) or internalResistorRatio.toUInt()
        writeData(false, data.toUByte(), false, true)
    }

    fun contrastPreciseSet(contrast: UByte) {
        val data = 0x70u or (contrast and 0x0Fu).toUInt()
        writeData(false, data.toUByte(), false, true)
    }

    override fun setDdRamAddress(address: UByte) {
        val data = 0x80u or (address and 0x7Fu).toUInt()
        writeData(false, data.toUByte(), false, null)
    }

    fun setScrollQuantity(quantity: Int) {
        require(quantity in 0..48) { "Scroll quantity must be between 0 and 48." }

        val data = 0x80u or quantity.toUInt()
        writeData(false, data.toUByte(), true, null)
    }

    fun temperatureCoefficientControl(tc: UByte) {
        require(tc in 0u..0b111u) { "Temperature coefficient must be between 0 and 7." }

        writeData(false, 0b01110110u, true, null)
        writeData(false, tc, null, null)
    }

    fun romSelection(romId: Int) {
        require(romId in 0 until 3) { "ROM ID must be 0 for ROM-A, 1 for ROM-B, and 2 for ROM-C." }

        writeData(false, 0b01110010u, true, null)
        writeData(false, (romId shl 2).toUByte(), null, null)
    }

    override fun writeData(rs: Boolean, data: UByte) {
        writeData(rs, data, false, false)
    }

    /**
     * @param reBit Forces the extended instruction set bit (RE) to be on or off.
     * @param isBit Forces the special registers bit (IS) to be on or off.
     */
    fun writeData(rs: Boolean, data: UByte, reBit: Boolean?, isBit: Boolean?)

    companion object {
        private const val NUL = '\u0000'

        val ROM_A = HD44780CharacterSet.of(
            NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL,
            '⯈', '⯇', '⏫', '⏬', '≪', '≫', '↖', '↗', '↙', '↘', '⯅', '⯆', '↵', '˄', '˅', '█',
            ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
            '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
            'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'Ä', 'Ö', 'Ñ', 'Ü', '§',
            '¿', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
            'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'ä', 'ö', 'ñ', 'ü', 'à',
            '⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹', '½', '¼', '±', '≥', '≤', 'μ',
            '♪', '♫', '⍾', '❤', '◆', '⊺', '「', '」', '“', '”', '（', '）', 'α', 'ε', 'δ', '∞',
            '@', '£', '$', '¥', 'è', 'é', 'ù', 'ì', 'ò', 'Ç', 'ᴾ', 'Ø', 'ø', 'ʳ', 'Ȧ', 'ȧ',
            '⌂', '¢', 'Φ', 'τ', 'λ', 'Ω', 'π', 'Ψ', 'Σ', '⟠', 'Ξ', '●', 'Æ', 'æ', 'ß', 'É',
            'Г', 'Л', 'П', '⥾', '_', 'È', NUL, NUL, 'ç', 'ǧ', 'Ş', 'ş', 'İ', 'ı', '~', '◇',
            NUL, NUL, NUL, NUL, NUL, '⨍', '█', '▊', '▌', '▎', '▏', '₧', '□', '⋅', '↑', '→',
            '↓', '←', 'Á', 'Í', 'Ó', 'Ú', 'Ý', 'á', 'í', 'ó', 'ú', 'ý', NUL, NUL, NUL, NUL,
            'Č', 'Ě', 'Ř', 'Š', 'Ž', 'č', 'ě', 'ř', 'š', 'ž', '[', '\\', ']', '{', '¦', '}',
        )

        val ROM_B = HD44780CharacterSet.of(
            NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL,
            '⯈', '⯇', '⏫', '⏬', '£', '¿', '¡', '♪', NUL, NUL, '✓', NUL, NUL, NUL, NUL, NUL,
            ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
            '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
            'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_',
            '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
            'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~', '┋',
            'А', 'Б', 'В', 'Г', 'Д', 'Е', 'Ж', 'З', 'И', 'Й', 'К', 'Л', 'М', 'Н', 'О', 'П',
            'Р', 'С', 'Т', 'У', 'Ф', 'Х', 'Ц', 'Ч', 'Ш', 'Щ', 'Ъ', 'Ы', 'Ь', 'Э', 'Ю', 'Я',
            'Ǎ', 'Ą', 'Ć', 'Č', 'Ď', 'Ě', 'Ę', 'Ğ', 'ğ', 'ı', 'İ', 'ł', 'Ń', 'Ň', 'Ő', 'Ř',
            'Ś', 'Ş', 'ş', 'Š', 'Ţ', 'Ť', NUL, 'Ű', 'Ź', 'Ż', 'Ž', NUL, NUL, NUL, NUL, NUL,
            'À', 'Á', 'Å', 'Ā', 'Ä', 'Ȧ', 'Æ', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ì', 'Í', 'Î', 'Ï',
            'Ð', 'Ñ', 'Ò', 'Ó', 'Ô', 'Õ', 'Ö', '×', 'Ø', 'Ù', 'Ú', 'Û', 'Ü', 'Ý', 'Þ', 'ß',
            'à', 'á', 'å', 'ā', 'ä', 'ȧ', 'æ', 'ç', 'è', 'é', 'ê', 'ë', 'ì', 'í', 'î', 'ï',
            'ð', 'ñ', 'ò', 'ó', 'ô', 'õ', 'ö', '÷', 'ø', 'ù', 'ú', 'û', 'ü', 'ý', 'þ', 'ÿ',
        )

        val ROM_C = HD44780CharacterSet.of(
            NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL,
            '™', '✝', '§', '¶', 'Г', 'Δ', 'Θ', 'Λ', 'Ξ', 'Π', 'Σ', '⥾', 'Φ', 'Ψ', 'Ω', '∞',
            ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
            '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
            'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '¥', ']', '^', '_',
            '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
            'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '→', '←',
            'Ç', 'ü', 'é', 'å', 'ä', 'à', 'ȧ', 'ç', NUL, 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å',
            'É', 'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù', 'ÿ', 'Ö', 'Ü', 'ñ', 'Ñ', 'ª', 'º', '¿',
            '\u3000', '。', '「', '」', '、', '・', 'ヲ', 'ァ', 'ィ', 'ゥ', 'ェ', 'ォ', 'ャ', 'ュ', 'ョ', 'ッ',
            'ー', 'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス', 'セ', 'ソ',
            'タ', 'チ', 'ツ', 'テ', 'ト', 'ナ', 'ニ', 'ヌ', 'ネ', 'ノ', 'ハ', 'ヒ', 'フ', 'ヘ', 'ホ', 'マ',
            'ミ', 'ム', 'メ', 'モ', 'ヤ', 'ユ', 'ヨ', 'ラ', 'リ', 'ル', 'レ', 'ロ', 'ワ', 'ン', '゛', '゜',
            'á', 'í', 'ó', 'ú', '¢', '£', '¥', '₧', 'ƒ', '¡', 'Ã', 'ã', 'Õ', 'õ', 'Ø', 'ø',
            '˙', '¨', '˚', 'ˋ', 'ˊ', '½', '¼', '×', '÷', '≤', '≥', '≪', '≫', '≠', '√', 'ˉ',
        )
    }
}
