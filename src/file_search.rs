use crate::test;
use std::path::PathBuf;
use std::{fs, io};

pub struct SearchArgs {
    pub folder: PathBuf,
    pub padding: u32,
    pub find: Option<PathBuf>,
}
pub fn file_search(
    SearchArgs {
        folder,
        padding,
        find,
    }: SearchArgs,
) -> Result<(), io::Error> {
    let files = fs::read_dir(folder)?;
    for file in files {
        let file = file?;
        match find.as_ref().and_then(|p| p.file_name()) {
            Some(name) if name == file.file_name() => {
                println!(
                    "Found {} at {}",
                    name.to_string_lossy(),
                    file.path().display()
                );
            }
            None => println!("{}", file.file_name().to_string_lossy()),
            _ => {}
        }

        if file.file_type()?.is_dir() {
            file_search(SearchArgs {
                folder: file.path(),
                padding,
                find: find.clone(),
            })?;
        }
    }

    Ok(())
}
