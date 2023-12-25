use std::path::Path;

extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        let major_version: u64 = env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap();
        let minor_version: u64 = env!("CARGO_PKG_VERSION_MINOR").parse().unwrap();
        let patch_version: u64 = env!("CARGO_PKG_VERSION_PATCH").parse().unwrap();
        let version: u64 = major_version << 48 | minor_version << 32 | patch_version << 16;
        let resource_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
        res.set_icon(resource_path.join("app_icon.ico").to_str().unwrap())
            .set_icon_with_id(
                resource_path.join("app_icon.ico").to_str().unwrap(),
                "app-icon",
            )
            .set("ProductName", "Genshin Paisitioning App")
            .set("InternalName", "Genshin Paisitioning App")
            .set("FileDescription", env!("CARGO_PKG_DESCRIPTION"))
            .set_version_info(winres::VersionInfo::PRODUCTVERSION, version);
        
            //res.set_manifest_file(resource_path.join("manifest.xml").to_str().unwrap());
            res.compile().unwrap();
    }
}
