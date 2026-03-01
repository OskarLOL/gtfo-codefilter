use std::env;
use std::path::PathBuf;

fn main() {
    // Only run this if we are compiling for Windows
    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        let mut res = winres::WindowsResource::new();
        
        // Get the absolute path to your project root
        let root = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut icon_path = PathBuf::from(root);
        
        // Match this exactly to where your .ico file is located
        // If it's in a folder called 'data' in your root:
        icon_path.push("data");
        icon_path.push("exec-brute-force.ico");

        if icon_path.exists() {
            res.set_icon(icon_path.to_str().unwrap());
        } else {
            // This will give you a helpful error message if the path is still wrong
            panic!("Icon not found at: {:?}", icon_path);
        }

        res.set("ProductName", "GTFO Terminal Decoder");
        res.set("FileDescription", "Utility for GTFO terminal codes");
        res.set("LegalCopyright", "Copyright © 2025 Oskar");
        
        res.compile().unwrap();
    }
}