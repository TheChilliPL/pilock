plugins {
    kotlin("multiplatform") version "1.9.23"
}

kotlin {
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
    }
}
