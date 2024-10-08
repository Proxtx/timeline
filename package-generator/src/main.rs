use {
    std::path::PathBuf,
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

    let mut file = File::open("main.Cargo.toml")
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
        .expect("Unable to write new Cargo.toml file");
}
