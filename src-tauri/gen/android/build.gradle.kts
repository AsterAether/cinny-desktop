buildscript {
    repositories {
        google()
        mavenCentral()
    }
    dependencies {
        classpath("com.android.tools.build:gradle:8.11.0")
        classpath("org.jetbrains.kotlin:kotlin-gradle-plugin:2.1.21")
    }
}

allprojects {
    repositories {
        google()
        mavenCentral()
    }
    configurations.all {
        resolutionStrategy.force(
            "org.jetbrains.kotlin:kotlin-stdlib:2.1.21",
            "org.jetbrains.kotlin:kotlin-stdlib-jdk7:2.1.21",
            "org.jetbrains.kotlin:kotlin-stdlib-jdk8:2.1.21"
        )
    }
}

tasks.register("clean").configure {
    delete("build")
}

