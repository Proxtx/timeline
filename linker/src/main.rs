use {
    std::path::PathBuf,
    tokio::{
        fs::{read_dir, write, File},
        io::AsyncReadExt,
    },
};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let enable_frontend = match args.get(1) {
        Some(v) => v == "enabled",
        None => true
    };
    let mut experiences_location = String::new();
    let mut experiences_location_file = File::open("../experiences_location.txt")
        .await
        .expect("Did not find experiences location file!");
    experiences_location_file
        .read_to_string(&mut experiences_location)
        .await
        .expect("Unable to read experiences location file!");

    let experiences_directory = PathBuf::from("../").join(PathBuf::from(experiences_location));

    let mut file = File::open("link.Cargo.toml")
        .await
        .expect("Did not find preset cargo file");
    let mut str = String::new();
    file.read_to_string(&mut str)
        .await
        .expect("Unable to read preset cargo file to string");

    let mut dirs = read_dir("../plugins/")
        .await
        .expect("Unable to find plugins directory");

    let mut plugins_str = String::new();
    let mut server_features_str = String::new();
    let mut client_features_str = String::new();

    while let Some(entry) = dirs
        .next_entry()
        .await
        .expect("Unable to read plugins directory")
    {
        let plugin_name = entry.file_name().into_string().expect("Unable to convert filename to string");
        let server_plugin_name = format!("{}_server", plugin_name);
        let client_plugin_name = format!("{}_client", plugin_name);
        server_features_str.push_str(&format!("\"dep:{}\", ", server_plugin_name));
        if enable_frontend {
            client_features_str.push_str(&format!("\"dep:{}\", ", client_plugin_name));
        }
        plugins_str.push_str(&format!("{0} = {{path=\"../plugins/{1}/server\", optional=true}}\n", server_plugin_name, plugin_name));
        if enable_frontend {
            plugins_str.push_str(&format!("{0} = {{path=\"../plugins/{1}/client\", optional=true}}\n", client_plugin_name, plugin_name));
        }
    }

    if enable_frontend {
        str += "experiences = [\"dep:experiences_navigator\"]\n";
        str += &format!("client = [{} \"client_api\"]\n", client_features_str);
    }
    str += &format!("server = [{} \"server_api\"]\n[dependencies]\n", server_features_str);
    str += &plugins_str;

    if enable_frontend {
        str += &format!(
            "\n
            experiences_navigator = {{path = \"{}\", optional = true}}
            \n",
            experiences_directory
                .join("experiences_navigator")
                .display(),
        );
    }

    str += "link_proc_macro = {path = \"../link_proc_macro/\"}\n";

    if enable_frontend {
        str += "client_api = {path = \"../client_api\", optional=true}\n";
    }
    str += "server_api = {path = \"../server_api\", optional=true}\n";

    write("../link/Cargo.toml", str)
        .await
        .expect("Unable to write new Cargo.toml file");
}
