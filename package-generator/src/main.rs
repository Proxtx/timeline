use {
    std::{fmt::format, path::PathBuf},
    tokio::{
        fs::{read_dir, try_exists, write, File},
        io::AsyncReadExt,
    },
};

#[tokio::main]
async fn main() {
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
        let server_plugin_name = format!("server_plugin_{}", plugin_name);
        let client_plugin_name = format!("client_plugin_{}", plugin_name);
        server_features_str.push_str(&format!("\"dep:{}\", ", server_plugin_name));
        client_features_str.push_str(&format!("\"dep:{}\", ", client_plugin_name));
        plugins_str.push_str(&format!("{0} = {{package = \"server\", path=\"../plugins/{1}/server\", optional=true}}\n", server_plugin_name, plugin_name));
        plugins_str.push_str(&format!("{0} = {{package = \"client\", path=\"../plugins/{1}/client\", optional=true}}\n", client_plugin_name, plugin_name));
    }

    str += &format!("server = [{}]\n", server_features_str);
    str += &format!("client = [{}]\n[dependencies]\n", client_features_str);

    str += &plugins_str;

    str += &format!(
        "\n
        experiences_navigator = {{path = \"{}\", optional = true}}
        \n",
        experiences_directory
            .join("experiences_navigator")
            .display(),
    );

    write("../link/Cargo.toml", str)
        .await
        .expect("Unable to write new Cargo.toml file");

    /*let mut file = File::open("main.Cargo.toml")
        .await
        .expect("Did not find preset cargo file");
    let mut str = String::new();
    file.read_to_string(&mut str)
        .await
        .expect("Unable to read preset cargo file to string");
    let mut dirs = read_dir("../plugins/")
        .await
        .expect("Unable to find plugins directory");
    while let Some(entry) = dirs
        .next_entry()
        .await
        .expect("Unable to read plugins directory")
    {
        let mut new_path = entry.path().to_path_buf();
        new_path.push("dependencies.txt");
        if try_exists(&new_path)
            .await
            .expect("Unable to check if dependencies file exists")
        {
            let mut cont = String::new();
            File::open(new_path)
                .await
                .expect("Unable to read dependencies txt file")
                .read_to_string(&mut cont)
                .await
                .expect("Unable to parse dependencies txt file");
            str.push_str(&format!("\n{}", cont));
        }
    }

    str.push_str(&format!(
        "\nexperiences_types={{path=\"{}\", optional = true}}",
        experiences_directory.join("experiences_types").display()
    ));

    write("../server/Cargo.toml", str)
        .await
        .expect("Unable to write new Cargo.toml file");

    //link frontend

    let mut file = File::open("frontend.Cargo.toml")
        .await
        .expect("Did not find preset cargo file");
    let mut str = String::new();
    file.read_to_string(&mut str)
        .await
        .expect("Unable to read preset cargo file to string");

    str += &format!(
        "\n
        experiences_navigator = {{path = \"{}\", optional = true}}
        \n",
        experiences_directory
            .join("experiences_navigator")
            .display(),
    );

    write("../frontend/Cargo.toml", str)
        .await
        .expect("Unable to write new Cargo.toml file");*/
}
