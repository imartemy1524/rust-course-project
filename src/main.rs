mod file_search;

use argh::FromArgs;
use crate::file_search::file_search;

#[derive(FromArgs)]
///
struct Arguments {
    /// whether or not to jump
    #[argh(positional, description = "folder to search")]
    name: String,
}

fn main() {
    let args: Arguments = argh::from_env();
    file_search(args.name.as_str()).unwrap();
}
