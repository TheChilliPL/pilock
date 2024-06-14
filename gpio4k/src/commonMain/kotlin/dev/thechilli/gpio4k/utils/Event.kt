package dev.thechilli.gpio4k.utils

class Event<T> {
    private val listeners = mutableListOf<(T) -> Unit>()

    fun subscribe(listener: (T) -> Unit) {
        listeners.add(listener)
    }

    fun unsubscribe(listener: (T) -> Unit) {
        listeners.remove(listener)
    }

    fun invoke(value: T) {
        listeners.forEach { it(value) }
    }
}
