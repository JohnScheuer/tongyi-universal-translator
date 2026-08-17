use std::path::Path;

fn main() {
    // Só faz sentido embedar resources quando o host é Windows (rc.exe disponível).
    let host = std::env::var("HOST").unwrap_or_default();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    let rc_path = Path::new("assets/tongyi.rc");
    let active_ico = Path::new("assets/tongyi_active.ico");
    let inactive_ico = Path::new("assets/tongyi_inactive.ico");

    // Só compila resources se:
    // - target é windows
    // - host é windows
    // - .rc existe
    // - os .ico existem (senão rc.exe falha)
    if target_os == "windows"
        && host.contains("windows")
        && rc_path.exists()
        && active_ico.exists()
        && inactive_ico.exists()
    {
        embed_resource::compile(rc_path, embed_resource::NONE);
        println!("cargo:rerun-if-changed=assets/tongyi.rc");
        println!("cargo:rerun-if-changed=assets/tongyi_active.ico");
        println!("cargo:rerun-if-changed=assets/tongyi_inactive.ico");
    } else {
        println!("cargo:warning=Skipping Windows resource embedding (host={host}, target_os={target_os}).");
        if rc_path.exists() {
            println!("cargo:rerun-if-changed=assets/tongyi.rc");
        }
        if active_ico.exists() {
            println!("cargo:rerun-if-changed=assets/tongyi_active.ico");
        }
        if inactive_ico.exists() {
            println!("cargo:rerun-if-changed=assets/tongyi_inactive.ico");
        }
    }
}
