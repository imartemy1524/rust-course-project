mod file_search;
mod sort;

use std::path::PathBuf;
use argh::FromArgs;
use crate::file_search::{file_search, SearchArgs};

#[derive(FromArgs)]
///
struct Arguments {
    /// whether or not to jump
    #[argh(positional, description = "folder to search")]
    folder: PathBuf,

    #[argh(option, description = "find")]
    find: Option<PathBuf>,

    #[argh(switch, description = "sort")]
    sort: bool,

}

fn main() {
    let args: Arguments = argh::from_env();
    let ans = file_search(SearchArgs{
        folder: &args.folder,
        find: args.find.as_ref(),
        sort: args.sort
    });
    match ans {
        Ok(value) => {
            for i in value{
                i.print(0)
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}
