fn main() {
    println!("cargo:rustc-link-arg=--import-memory");
    println!("cargo:rustc-link-arg=-zstack-size=0");
}