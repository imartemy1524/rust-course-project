use std::{fs, io};

pub fn file_search(folder: &str) -> Result<(), io::Error> {
    let files = fs::read_dir(folder)?;


    for file in files {
        let file = file?;

        println!("{}", file.file_name().to_string_lossy());
    }


    Ok(())
}