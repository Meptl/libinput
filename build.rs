extern crate bindgen;

// Generate rust bindings to libinput library
fn generate_libinput_bindings() {
    let mut builder = bindgen::Builder::new("src/ffi/cc/libinput.c");
    builder.link("input", bindgen::LinkType::Dynamic);
    builder.link("udev", bindgen::LinkType::Dynamic);
    builder.builtins();
    match builder.generate() {
        Ok(b) => b.write_to_file("src/ffi/libinput_c.rs").unwrap(),
        Err(e) => panic!(e)
    };
}

pub fn main() {
    generate_libinput_bindings();
}
