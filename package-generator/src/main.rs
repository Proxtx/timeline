use tokio::{
    fs::{read_dir, try_exists, write, File},
    io::AsyncReadExt,
};

#[tokio::main]
async fn main() {
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

    write("../server/Cargo.toml", str)
        .await
        .expect("Unable to write new Cargo.toml file");
}
