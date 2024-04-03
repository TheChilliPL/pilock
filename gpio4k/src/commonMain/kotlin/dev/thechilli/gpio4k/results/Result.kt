package dev.thechilli.gpio4k.results

sealed class Result<out TSuccess, out TFailure, out TNotice>(val notices: List<TNotice>) {
    abstract val isSuccess: Boolean
    val isFailure: Boolean get() = !isSuccess

    abstract fun getOrNull(): TSuccess?
    abstract fun getOrThrow(): TSuccess

    abstract fun getFailureOrNull(): TFailure?

    abstract fun <T> map(transform: (TSuccess) -> T): Result<T, TFailure, TNotice>
    abstract fun <T> mapFailure(transform: (TFailure) -> T): Result<TSuccess, T, TNotice>
    abstract fun <T> mapNotices(transform: (TNotice) -> T): Result<TSuccess, TFailure, T>

    class Success<out TSuccess, out TNotice> private constructor(val value: TSuccess, notices: List<TNotice>) :
        Result<TSuccess, Nothing, TNotice>(notices) {
        override val isSuccess: Boolean = true

        override fun getOrNull(): TSuccess = value
        override fun getOrThrow(): TSuccess = value

        override fun getFailureOrNull(): Nothing? = null

        override fun <T> map(transform: (TSuccess) -> T): Success<T, TNotice> = Success(transform(value), notices)
        override fun <T> mapFailure(transform: (Nothing) -> T): Success<TSuccess, TNotice> = Success(value, notices)
        override fun <T> mapNotices(transform: (TNotice) -> T): Success<TSuccess, T> = Success(value, notices.map(transform))

        companion object {
            fun <TSuccess> of(value: TSuccess): Success<TSuccess, Nothing> = Success(value, emptyList())
            fun <TSuccess, TNotice> of(value: TSuccess, vararg notices: TNotice): Success<TSuccess, TNotice> = Success(value, notices.toList())
        }
    }

    class Failure<out TFailure, out TNotice> private constructor(val error: TFailure, notices: List<TNotice>) :
        Result<Nothing, TFailure, TNotice>(notices) {
        override val isSuccess: Boolean = false

        override fun getOrNull(): Nothing? = null
        override fun getOrThrow(): Nothing = throw NoSuchElementException("Result is failure")

        override fun getFailureOrNull(): TFailure = error

        override fun <T> map(transform: (Nothing) -> T): Failure<TFailure, TNotice> = Failure(error, notices)
        override fun <T> mapFailure(transform: (TFailure) -> T): Failure<T, TNotice> = Failure(transform(error), notices)
        override fun <T> mapNotices(transform: (TNotice) -> T): Failure<TFailure, T> = Failure(error, notices.map(transform))

        companion object {
            fun <TFailure> of(error: TFailure): Failure<TFailure, Nothing> = Failure(error, emptyList())
            fun <TFailure, TNotice> of(error: TFailure, vararg notices: TNotice): Failure<TFailure, TNotice> = Failure(error, notices.toList())
        }
    }
}
