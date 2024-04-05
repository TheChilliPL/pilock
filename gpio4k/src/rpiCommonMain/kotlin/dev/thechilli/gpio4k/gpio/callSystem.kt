package dev.thechilli.gpio4k.gpio

/**
 * Executes a command, wait for it to end, and returns the exit code and output.
 */
expect fun exec(command: String, vararg args: String): Pair<Int, String>

/**
 * Executes a command in the background and returns the process id.
 */
expect fun spawn(command: String, vararg args: String): Long

fun kill(pid: Long) {
    exec("kill", "-s", "SIGTERM", pid.toString())
}
