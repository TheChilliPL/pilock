package dev.thechilli.gpio4k.gpio

import kotlinx.cinterop.*
import platform.posix.*

private var setupForksDone = false

/**
 * Sets up the signal handler for forked processes.
 * Prevents zombie processes.
 * This function is idempotent.
 */
private inline fun setupForks() {
    if(setupForksDone) return

    // Ignore SIGCHLD to avoid zombie processes
    signal(SIGCHLD, SIG_IGN)

    setupForksDone = true
}

/**
 * Executes a command, wait for it to end, and returns the exit code and output.
 */
actual fun exec(command: String, vararg args: String): Pair<Int, String> = memScoped {
    setupForks()

    // Allocate array for file descriptors of the pipe
    val fds = allocArray<IntVar>(2)
    // Create the pipe and set fds to the file descriptors
    val pipe = pipe(fds)
    if (pipe != 0) {
        throw GpioException("Failed to create pipe")
    }
    // Create a child process
    val pid = fork()
    if (pid < 0) {
        throw GpioException("failed to fork. errno: $errno")
    } else if(pid == 0) {
        // Forked process
        // We don't need the read end of the pipe in the child process, so we close it
        close(fds[0])
        // Redirect stdout and stderr to the write end of the pipe
        dup2(fds[1], STDOUT_FILENO)
        dup2(fds[1], STDERR_FILENO)
        // Close the write end of the pipe
        close(fds[1])

        // Execute the command
        val commandArray = allocArrayOf((listOf(command) + args.toList() + null).map { it?.cstr?.ptr })
        execvp(command, commandArray)
        // If execvp returns, it failed
        perror("execvp failed. errno: $errno")
        exit(1)
    }

    // Parent process
    // Close the write end of the pipe
    close(fds[1])
    // Read the output from the read end of the pipe
    // We start with a buffer of 4096 bytes
    val blockSize = 1024
    var buffer = ByteArray(blockSize)
    var currentIndex = 0
    while(read(fds[0], buffer.refTo(currentIndex * blockSize), blockSize.toULong()) > 0) {
        currentIndex++
        buffer = buffer.copyOf((currentIndex + 1) * blockSize)
    }
    // We wait for the child process to end
    val status = alloc<IntVar>()
    waitpid(pid, status.ptr, 0)
    return Pair(status.value, buffer.decodeToString())
}

/**
 * Executes a command in the background and returns the process id.
 */
actual fun spawn(command: String, vararg args: String): Long = memScoped {
    setupForks()

    // Create a child process
    val pid = fork()
    if (pid < 0) {
        throw GpioException("failed to fork. errno: $errno")
    } else if(pid == 0) {
        // Forked process
        // Execute the command
        val commandArray = allocArrayOf((listOf(command) + args.toList() + null).map { it?.cstr?.ptr })
        execvp(command, commandArray)
        // If execvp returns, it failed
        perror("execvp failed. errno: $errno")
        exit(1)
    }
    return pid.toLong()
}
