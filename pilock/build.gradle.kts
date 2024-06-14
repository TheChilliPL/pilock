import org.jetbrains.kotlin.gradle.ExperimentalKotlinGradlePluginApi
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    kotlin("multiplatform") version "1.9.23"
}

kotlin {
    @OptIn(ExperimentalKotlinGradlePluginApi::class)
    compilerOptions {
        optIn.add("kotlin.ExperimentalStdlibApi")
    }

    val targetAttr = Attribute.of("target", String::class.java)

    linuxArm64("rpiNative") {
        binaries {
            executable()
        }
        attributes.attribute(targetAttr, "rpi")
    }

    jvm("rpiJvm") {
        withJava()
        compilations.getting {
            compilerOptions.configure {
                jvmTarget.set(JvmTarget.JVM_17)

            }
        }
        @OptIn(ExperimentalKotlinGradlePluginApi::class)
        mainRun {
            this.mainClass = "dev.thechilli.pilock.MainKt"
        }
        attributes.attribute(targetAttr, "rpi")
    }

    jvm("desktopJvm") {
        @OptIn(ExperimentalKotlinGradlePluginApi::class)
        mainRun {
            this.mainClass = "dev.thechilli.pilock.MainKt"
        }
        attributes.attribute(targetAttr, "desktop")
    }

    mingwX64("desktopNative") {
        binaries {
            executable()
        }
        attributes.attribute(targetAttr, "desktop")
    }

    sourceSets {
        val commonMain by getting {
            dependencies {
                implementation(project(":gpio4k"))
            }
        }

        val rpiCommonMain by creating {
            dependsOn(commonMain)
        }

        val rpiNativeMain by getting {
            dependsOn(rpiCommonMain)
        }

        val rpiJvmMain by getting {
            dependsOn(rpiCommonMain)
        }

        val desktopCommonMain by creating {
            dependsOn(commonMain)
        }

        val desktopJvmMain by getting {
            dependsOn(desktopCommonMain)
        }

        val desktopNativeMain by getting {
            dependsOn(desktopCommonMain)
        }
    }
}
