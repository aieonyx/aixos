// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
fn main() {
    let target = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target != "aarch64" { return; }

    // Manifest dir = crates/aixos-init/ — boot/ is two levels up
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = std::path::Path::new(&manifest).join("../..").canonicalize().unwrap();
    let boot_s   = root.join("boot/boot.s");
    let head_s   = root.join("boot/head_pe.s");

    println!("cargo:rerun-if-changed={}", boot_s.display());
    println!("cargo:rerun-if-changed={}", head_s.display());

    let out = std::env::var("OUT_DIR").unwrap();

    let boot_obj = format!("{}/boot.o", out);
    let s1 = std::process::Command::new("aarch64-linux-gnu-as")
        .args(["-o", &boot_obj, boot_s.to_str().unwrap()])
        .status().unwrap();
    if s1.success() {
        println!("cargo:rustc-link-arg={}", boot_obj);
    }

    let head_obj = format!("{}/head_pe.o", out);
    let s2 = std::process::Command::new("aarch64-linux-gnu-as")
        .args(["-o", &head_obj, head_s.to_str().unwrap()])
        .status().unwrap();
    if s2.success() {
        println!("cargo:rustc-link-arg={}", head_obj);
    }

    // Strip debug — prevents .eh_frame becoming LOAD sections that corrupt PE layout
    println!("cargo:rustc-link-arg=--strip-debug");
}
