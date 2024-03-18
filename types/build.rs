use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../plugins/");
    println!("cargo:rerun-if-changed=build.rs");
    let mut out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    out_path = out_path.join("out");
    out_path.set_file_name("plugins.rs");
    let plugins: Vec<String> = fs::read_dir("../plugins")
        .expect("Plugins Folder not found.")
        .map(|v| {
            let entry = v.expect("Unable to read dir entry");
            entry.file_name().into_string().unwrap()
        })
        .collect();
    let enum_str = plugins.join(",");

    let file = format!(
        "
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
    #[allow(non_camel_case_types)]
    pub enum AvailablePlugins {{
        {}
    }}

    impl fmt::Display for AvailablePlugins {{
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
            write!(f, \"{{:?}}\", self)
        }}
    }}",
        enum_str
    );

    fs::write(out_path, file).expect("Unable to write plugins file");
}
