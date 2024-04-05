import org.jetbrains.kotlin.gradle.ExperimentalKotlinGradlePluginApi
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    kotlin("multiplatform") version "1.9.23"
}

kotlin {
    @OptIn(ExperimentalKotlinGradlePluginApi::class)
    compilerOptions {
        optIn.apply{
            // Experimental Kotlin APIs
            add("kotlin.ExperimentalStdlibApi")
            add("kotlin.ExperimentalUnsignedTypes")
            add("kotlinx.cinterop.ExperimentalForeignApi")
        }
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
        compilations.getting {
            compilerOptions.configure {
                jvmTarget.set(JvmTarget.JVM_17)
            }
        }
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
