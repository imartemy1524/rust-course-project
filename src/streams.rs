use std::fs::File;
use std::io;
use std::io::{Error, Write};
use std::path::PathBuf;

#[inline]
pub fn open_file(f: Option<PathBuf>) -> Result<Box<dyn Write>, Error> {
    let writer: Box<dyn Write> = match f {
        Some(file) => {
            let ans = File::create(file)?;
            Box::new(io::BufWriter::new(ans))
        }
        None => Box::new(io::BufWriter::new(io::stdout())),
    };
    Ok(writer)
}
