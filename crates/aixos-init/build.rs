fn main() {
    println!("cargo:rerun-if-changed=../../boot/boot.s");
    let out = std::env::var("OUT_DIR").unwrap();
    let status = std::process::Command::new("aarch64-linux-gnu-as")
        .args(["-o", &format!("{}/boot.o", out), "../../boot/boot.s"])
        .status().unwrap();
    assert!(status.success());
    println!("cargo:rustc-link-arg={}/boot.o", out);
}
