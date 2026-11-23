import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.yueli.bdl.media"
    compileSdk = 37

    defaultConfig {
        minSdk = 24
        consumerProguardFiles("consumer-rules.pro")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    packaging {
        resources {
            excludes += "META-INF/native-image/**"
        }
    }
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_1_8
    }
}

dependencies {
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation(project(":tauri-android"))
    implementation("org.bytedeco:javacpp:1.5.14")
    implementation("org.bytedeco:javacv:1.5.14") { isTransitive = false }
    implementation("org.bytedeco:javacpp:1.5.14:android-arm64")
    implementation("org.bytedeco:ffmpeg:8.1.2-1.5.14")
    implementation("org.bytedeco:ffmpeg:8.1.2-1.5.14:android-arm64")
    testImplementation("com.fasterxml.jackson.core:jackson-databind:2.15.3")
    testImplementation("junit:junit:4.13.2")
}
