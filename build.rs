use std::{env, path::Path};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let rc_path = "resources/tongyi.rc";
    println!("cargo:rerun-if-changed={}", rc_path);

    if Path::new(rc_path).exists() {
        embed_resource::compile(rc_path, embed_resource::NONE);
    } else {
        println!("cargo:warning=Skipping Windows resource embedding (missing {}).", rc_path);
    }
}
