package dev.thechilli.gpio4k.gpio

/**
 * Executes a command, wait for it to end, and returns the exit code and output.
 */
actual fun exec(command: String, vararg args: String): Pair<Int, String> {
    val process = ProcessBuilder(command, *args).start()
    val str = process.inputStream.bufferedReader().readText()
    return Pair(process.waitFor(), str)
}

/**
 * Executes a command in the background and returns the process id.
 */
actual fun spawn(command: String, vararg args: String): Long {
    val process = ProcessBuilder(command, *args).start()
    return process.pid()
}
