fn main() {
    let windows =
        tauri_build::WindowsAttributes::new().app_manifest(include_str!("app.manifest.xml"));
    let attributes = tauri_build::Attributes::new().windows_attributes(windows);

    if let Err(error) = tauri_build::try_build(attributes) {
        eprintln!("failed to build WubiLex application resources: {error:#}");
        std::process::exit(1);
    }
}
