#![feature(let_chains)]

use {
    std::{env, path::PathBuf},
    stylers::build,
};
fn main() {
    println!("cargo:rerun-if-changed=src/");
    let mut style_path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    while let Some(name) = style_path.file_name()
        && name != "target"
    {
        style_path.pop();
    }
    style_path.push("generated.css");
    build(Some(style_path.display().to_string()));
}
