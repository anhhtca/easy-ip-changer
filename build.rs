fn main() {
    println!("cargo:rerun-if-changed=ui/");
    println!("cargo:rerun-if-changed=assets/app.manifest");
    println!("cargo:rerun-if-changed=assets/app.ico");
    println!("cargo:rerun-if-changed=assets/app-icon.png");

    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-light".into());
    slint_build::compile_with_config("ui/app.slint", config)
        .expect("Không thể biên dịch giao diện Slint");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_manifest_file("assets/app.manifest");
        res.set_icon("assets/app.ico");
        res.set("ProductName", "Easy IP Changer");
        res.set("FileDescription", "Công cụ đổi nhanh địa chỉ IP cho Windows 10/11 Pro");
        res.set("LegalCopyright", "Easy IP Changer");
        res.compile().expect("Không thể nhúng Windows manifest/resource");
    }
}
