import org.jetbrains.kotlin.gradle.ExperimentalKotlinGradlePluginApi

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
            // Library
            sharedLib()
        }
        attributes.attribute(targetAttr, "rpi")
    }

    jvm("rpiJvm") {
        attributes.attribute(targetAttr, "rpi")
    }

    jvm("desktopJvm") {
        attributes.attribute(targetAttr, "desktop")
    }

    sourceSets {
        val commonMain by getting {
            dependencies {
                // ...
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

        val desktopJvmMain by getting {
            dependsOn(commonMain)
        }

        val desktopJvmTest by getting {
            dependencies {
                implementation(kotlin("test"))
                implementation(kotlin("test-junit"))
            }
        }
    }
}
