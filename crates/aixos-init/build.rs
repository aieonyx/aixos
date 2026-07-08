// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
fn main() {
    let target = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target != "aarch64" { return; }

    println!("cargo:rerun-if-changed=../../boot/boot.s");
    println!("cargo:rerun-if-changed=../../boot/head_pe.s");

    let assembler = std::process::Command::new("aarch64-linux-gnu-as")
        .arg("--version")
        .status();
    if assembler.is_err() { return; }

    let out = std::env::var("OUT_DIR").unwrap();

    let boot_obj = format!("{}/boot.o", out);
    let s1 = std::process::Command::new("aarch64-linux-gnu-as")
        .args(["-o", &boot_obj, "../../boot/boot.s"])
        .status().unwrap();
    if s1.success() {
        println!("cargo:rustc-link-arg={}", boot_obj);
    }

    let head_obj = format!("{}/head_pe.o", out);
    let s2 = std::process::Command::new("aarch64-linux-gnu-as")
        .args(["-o", &head_obj, "../../boot/head_pe.s"])
        .status().unwrap();
    if s2.success() {
        println!("cargo:rustc-link-arg={}", head_obj);
    }
}
