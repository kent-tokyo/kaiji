plugins {
    java
}

group = "io.github.kenttokyo"
version = "0.1.0"

repositories {
    mavenCentral()
}

dependencies {
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.0")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

tasks.test {
    useJUnitPlatform()
    // Native library path: set via -Dkaiji.lib.path=<path>
    systemProperty("kaiji.lib.path", System.getProperty("kaiji.lib.path", ""))
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(17))
    }
}
