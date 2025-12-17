fn main() {    
    // If you want to force x86 compilation, use this:
    println!("cargo:rustc-check-cfg=cfg(target, values(\"i686-pc-windows-msvc\"))");
    println!("cargo:rustc-cdylib-link-arg=/DEF:d3d9.def");
}