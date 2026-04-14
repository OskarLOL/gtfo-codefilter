use std::{env, path::PathBuf};

fn main() {
    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        let mut res = winres::WindowsResource::new();

        // Tell winres to use the mingw windres when cross-compiling
        res.set_windres_path("x86_64-w64-mingw32-windres");
        res.set_ar_path("x86_64-w64-mingw32-ar");

        let root = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut icon_path = PathBuf::from(root);
        icon_path.push("data");
        icon_path.push("exec-brute-force.ico");

        if icon_path.exists() {
            res.set_icon(icon_path.to_str().unwrap());
        }

        res.set("ProductName", "GTFO Code Filter");
        res.set("FileDescription", "Utility for GTFO terminal codes");
        res.set("LegalCopyright", "Copyright © 2025 Oskar");
        res.compile().unwrap();
    }
}
