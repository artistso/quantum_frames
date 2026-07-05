plugins {
    id("com.android.application")
    kotlin("android")
}

android {
    namespace = "com.quantumframes"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.quantumframes"
        minSdk = 30
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0"
        ndk {
            abiFilters += listOf("arm64-v8a", "x86_64")
        }
    }

    buildFeatures {
        compose = true
    }
    composeOptions {
        kotlinCompilerExtensionVersion = "1.5.8"
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = "17"
    }
}

// ----- Rust build tasks -----
tasks.register<Exec>("buildRustDebug") {
    workingDir = file("${rootDir}/rust")
    commandLine("cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "x86_64",
        "-o", "${projectDir}/src/main/jniLibs",
        "build"
    )
}
tasks.register<Exec>("buildRustRelease") {
    workingDir = file("${rootDir}/rust")
    commandLine("cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "x86_64",
        "--release",
        "-o", "${projectDir}/src/main/jniLibs",
        "build"
    )
}
afterEvaluate {
    tasks.named("preDebugBuild") { dependsOn("buildRustDebug") }
    tasks.named("preReleaseBuild") { dependsOn("buildRustRelease") }
}

dependencies {
    implementation("androidx.core:core-ktx:1.12.0")
    implementation("androidx.activity:activity-compose:1.8.2")
    implementation(platform("androidx.compose:compose-bom:2024.01.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.ui:ui-tooling-preview")
}
