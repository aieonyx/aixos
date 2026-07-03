// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
fn main() {
    println!("cargo:rerun-if-changed=../../boot/boot.s");
    let assembler = std::process::Command::new("aarch64-linux-gnu-as")
        .arg("--version")
        .status();
    if assembler.is_err() { return; }
    let out = std::env::var("OUT_DIR").unwrap();
    let obj = format!("{}/boot.o", out);
    let status = std::process::Command::new("aarch64-linux-gnu-as")
        .args(["-o", &obj, "../../boot/boot.s"])
        .status().unwrap();
    if status.success() {
        println!("cargo:rustc-link-arg={}", obj);
    }
}
