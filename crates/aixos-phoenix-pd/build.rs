// Copyright (c) 2026 Edison Lepiten / AIEONYX
fn main() {
    let sdk = "/home/edisonbl/aixos/microkit-sdk-1.4.1/board/qemu_virt_aarch64/release/lib";
    let ld = "/home/edisonbl/aixos/microkit-phoenix.ld";
    println!("cargo:rustc-link-arg=-T{ld}");
    println!("cargo:rustc-link-arg=-L{sdk}");
    println!("cargo:rustc-link-arg=-lmicrokit");
    println!("cargo:rustc-link-arg=--undefined=microkit_name");
    println!("cargo:rustc-link-arg=--undefined=microkit_passive");
    println!("cargo:rustc-link-arg=--undefined=__sel4_ipc_buffer_obj");
    println!("cargo:rustc-link-arg=--undefined=init");
    println!("cargo:rustc-link-arg=--undefined=notified");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=/home/edisonbl/aixos/microkit-phoenix.ld");
}
