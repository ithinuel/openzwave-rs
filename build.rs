extern crate cc;
fn main() {
    cc::Build::new()
        .warnings(false)
        .include("/usr/include/openzwave")
        .file("src/c/wrapper.cpp")
        .compile("wrapper");
}
