fn main() -> std::io::Result<()> {
    if cfg!(target_os = "windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/satellite_icon_262671.ico")
            .set("InternalName", "TC-UTILS.EXE")
            // manually set version 1.0.0.0
            .set_version_info(winresource::VersionInfo::PRODUCTVERSION, 0x0001000000000000);
        res.compile()?;
    }
    println!("cargo::rerun-if-changed=build.rs");
    Ok(())
}
