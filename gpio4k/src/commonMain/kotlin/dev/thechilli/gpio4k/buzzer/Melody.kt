package dev.thechilli.gpio4k.buzzer

class Melody {
    private val _notes = mutableListOf<Note>()
    val notes: List<Note> = _notes

    fun add(note: Note) {
        _notes.add(note)
    }

    companion object {
        fun of(vararg notes: Note) = Melody().apply {
            notes.forEach { add(it) }
        }
    }
}
