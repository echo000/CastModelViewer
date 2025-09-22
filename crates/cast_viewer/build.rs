#[cfg(target_os = "windows")]
fn main() {
    use porter_build::configure_windows;

    configure_windows("cast.ico", false).expect("unable to compile Windows resource");
    println!("cargo:rerun-if-changed=build.rs");
}

#[cfg(not(target_os = "windows"))]
fn main() {}
