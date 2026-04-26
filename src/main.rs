mod file_search;
mod test;

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
    find: Option<PathBuf>
}

fn main() {
    let args: Arguments = argh::from_env();
    file_search(SearchArgs{
        padding: 0u32,
        folder: args.folder,
        find: args.find,
    }).unwrap();
}
