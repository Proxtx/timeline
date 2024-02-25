#![feature(iter_intersperse)]
use std::{env, fmt::Write, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=plugins/");
    println!("cargo:rerun-if-changed=build.rs");
    let mut out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    out_path = out_path.join("out");
    out_path.set_file_name("plugins.rs");
    let plugins: Vec<(String, String)> = fs::read_dir("plugins")
        .expect("Plugins Folder not found.")
        .map(|v| {
            let dir_entry = v.expect("unable to read directory");
            if dir_entry
                .file_type()
                .expect("unable to read file-type")
                .is_file()
            {
                panic!("Did not expect a file in plugins folder");
            }
            let name = dir_entry
                .file_name()
                .into_string()
                .expect("unable to parse filename");
            let mut path = dir_entry.path();
            path.push("plugin.rs");
            (
                name,
                fs::canonicalize(&path)
                    .expect("unable to resolve path")
                    .into_os_string()
                    .into_string()
                    .expect("os string error"),
            )
        })
        .collect();
    let mod_str = plugins.iter().fold(String::new(), |mut output, b| {
        let _ = write!(
            output,
            "
        #[path = \"{}\"]
        mod {};",
            b.1.replace('\\', "\\\\").replace('\"', "\\\""),
            b.0
        );
        output
    });
    let as_enum = plugins
        .iter()
        .map(|v| v.0.to_string())
        .intersperse(String::from(','))
        .collect::<String>();
    let init_str = plugins
        .iter()
        .map(|v| {
            format!(
                "(\"{}\".to_string(), Box::new({}::Plugin::new(handler(AvailablePlugins::{})).await) as Box<dyn Plugin>)",
                v.0, v.0, &v.0
            )
        })
        .intersperse(", ".to_string())
        .collect::<String>();
    let importer = format!(
        "
    //dynamic module imports
    {}

    use {{
        serde::{{Serialize, Deserialize}},
        std::{{fmt, collections::HashMap}}
    }};
    
    pub struct Plugins<'a> {{
        pub plugins: HashMap<String, Box<dyn Plugin<'a> + 'a>>
    }}

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
    #[allow(non_camel_case_types)]
    pub enum AvailablePlugins {{
        {}
    }}

    impl fmt::Display for AvailablePlugins {{
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
            write!(f, \"{{:?}}\", self)
        }}
    }}

    impl<'a> Plugins<'a> {{
        pub async fn init(handler: impl Fn(AvailablePlugins) -> PluginData<'a>) -> Plugins<'a> {{
            Plugins {{
                plugins: HashMap::from([{}])
            }}
        }}
    }}
    ",
        mod_str, as_enum, init_str
    );
    fs::write(out_path, importer).expect("Unable to write plugins file");
}
