fn main() {
    tauri_plugin::Builder::new(&[])
        .android_path("android")
        .try_build()
        .expect("failed to build Android mobile media plugin");
}
