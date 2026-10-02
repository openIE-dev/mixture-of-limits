fn main() {
    // IOReport is a private dylib (libIOReport.tbd in the macOS SDK), not a
    // .framework bundle on current macOS. Link it only for the energy-meter feature.
    let macos = std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() == Some("macos");
    let feature = std::env::var_os("CARGO_FEATURE_ENERGY_METER").is_some();
    if macos && feature {
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=IOReport");
        if let Ok(out) = std::process::Command::new("xcrun")
            .args(["--show-sdk-path"])
            .output()
        {
            if out.status.success() {
                let sdk = String::from_utf8_lossy(&out.stdout);
                let sdk = sdk.trim();
                if !sdk.is_empty() {
                    println!("cargo:rustc-link-search=native={sdk}/usr/lib");
                }
            }
        }
    }
}
