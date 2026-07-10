fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let bindings = cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("Unable to generate C bindings");
    bindings.write_to_file("inky.h");

    // The PHP stub is the only header tracked in git; keep it in sync with
    // the real ABI so FFI::cdef never calls with a stale signature.
    let stub = std::path::Path::new(&crate_dir).join("../../bindings/php/stubs/inky.h");
    if stub.parent().is_some_and(|p| p.exists()) {
        bindings.write_to_file(stub);
    }
}
