mod file_search;
mod sort;
mod streams;

use crate::file_search::{SearchArgs, file_search};
use crate::streams::open_file;
use argh::FromArgs;
use std::fs::File;
use std::io;
use std::io::{Error, Write};
use std::path::PathBuf;
use std::process::{exit, Termination};

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

    #[argh(option, description = "output file")]
    f: Option<PathBuf>,
}

#[inline]
fn run(args: Arguments) -> io::Result<()> {
    let ans = file_search(SearchArgs {
        folder: &args.folder,
        find: args.find.as_ref(),
        sort: args.sort,
    })?;
    let mut output = open_file(args.f)?;
    for i in ans {
        i.print(0, &mut output)?
    }
    Ok(())
}
fn main() {
    let args: Arguments = argh::from_env();
    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        exit(1);
    }
}
