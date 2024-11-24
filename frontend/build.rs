use {
    std::{env, path::PathBuf},
    stylers::build,
};
fn main() {
    println!("cargo:rerun-if-changed=src/");
    let style_path =
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("../../../../generated.css");
    build(Some(style_path.display().to_string()));
}
