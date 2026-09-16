fn main() {
    // The library test harness also links Tauri's native dialog dependencies.
    // Apply Common Controls v6 to every executable, including cargo test.
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest()),
    )
    .expect("Tauri build failed");
    println!("cargo:rerun-if-changed=windows.rc");
    println!("cargo:rerun-if-changed=windows.manifest");
    embed_resource::compile_for_everything("windows.rc", embed_resource::NONE)
        .manifest_required()
        .expect("Windows manifest compilation failed");
}
