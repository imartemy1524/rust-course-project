use crate::sort::mergesort;
use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::LinkedList;
use std::io::{Error, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::{fs, io};

pub struct SearchArgs<'c> {
    pub folder: &'c PathBuf,
    pub find: Option<&'c PathBuf>,
    pub sort: bool,
    pub in_file: Option<&'c String>,
}

enum Occurrence {
    File(Rc<PathBuf>),
    Directory(SmartPath),
    TextFile(Rc<PathBuf>),
}

pub struct SmartPath {
    fname: Rc<PathBuf>,
    children: Rc<Cell<Vec<SmartPath>>>,
}
impl SmartPath {
    fn new(string: PathBuf) -> SmartPath {
        SmartPath {
            fname: Rc::new(string),
            children: Rc::new(Cell::new(Vec::with_capacity(0))),
        }
    }
    pub(crate) fn print(&self, padding: usize, file: &mut Box<dyn Write>) -> io::Result<()> {
        let items = self.children.take();
        let name = self.fname.to_string_lossy();
        writeln!(
            file,
            "{}{}{}",
            " ".repeat(padding),
            name,
            if items.is_empty() { "" } else { "/" }
        )?;

        for child in items.iter() {
            child.print(padding + name.len() + 1, file)?
        }
        self.children.set(items);
        Ok(())
    }
}

impl Clone for SmartPath {
    fn clone(&self) -> Self {
        SmartPath {
            fname: self.fname.clone(),
            children: self.children.clone(),
        }
    }
}

impl Eq for SmartPath {}

impl PartialEq<Self> for SmartPath {
    fn eq(&self, other: &Self) -> bool {
        self.fname == other.fname
    }
}

impl PartialOrd<Self> for SmartPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let ans = self.cmp(other);
        Some(ans)
    }
}

impl Ord for SmartPath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.fname.as_ref().cmp(other.fname.as_ref())
    }
}

pub fn file_search(
    SearchArgs {
        folder,
        find,
        sort,
        in_file,
    }: SearchArgs,
) -> Result<Vec<SmartPath>, io::Error> {
    let mut ans: LinkedList<Occurrence> = LinkedList::new();

    let files = fs::read_dir(folder)?;
    for file in files {
        let file = file?;
        let value = if file.file_type()?.is_dir() {
            let path = SmartPath::new(file.path());
            let recursive = file_search(SearchArgs {
                folder: &file.path(),
                find,
                sort,
                in_file,
            })?;
            path.children.replace(recursive);
            Occurrence::Directory(path)
        } else if file.file_name().to_string_lossy().ends_with(".txt")
            || file.file_name().to_string_lossy().ends_with(".rs")
        {
            Occurrence::TextFile(Rc::new(file.path()))
        } else {
            Occurrence::File(Rc::new(file.path()))
        };

        ans.push_back(value);
    }

    let mut answer: Vec<SmartPath> = Vec::new();
    for i in ans {
        if should_include(&i, &in_file, &find)? {
            match i {
                Occurrence::File(i) | Occurrence::TextFile(i) => {
                    let path = (*i).clone();
                    answer.push(SmartPath::new(path));
                }
                Occurrence::Directory(i) => {
                    answer.push(i);
                }
            };
        }
    }
    if sort {
        mergesort(&mut answer);
    }
    Ok(answer)
}

#[inline]
fn should_include<'c>(
    item: &Occurrence,
    in_file: &Option<&'c String>,
    find: &Option<&'c PathBuf>,
) -> Result<bool, Error> {
    if let Some(in_file) = in_file {
        if let Occurrence::TextFile(item) = item {
            let contents = fs::read_to_string(item.as_ref());
            match contents {
                Err(_) => {
                    let q = item.to_string_lossy();
                    eprintln!("Error while reading file {q}");
                    return Ok(false);
                }
                Ok(contents) => {
                    if !contents.contains(*in_file) {
                        return Ok(false);
                    }
                }
            }
        }
        else{
            return Ok(false);
        }
    }
    if let Some(find) = find.as_ref().and_then(|p| p.file_name()) {
        let name = match item {
            Occurrence::File(q) | Occurrence::TextFile(q) => q.as_ref().file_name(),
            Occurrence::Directory(q) => q.fname.as_ref().file_name(),
        };
        match name {
            None => return Ok(false),
            Some(name) if name != find => return Ok(false),
            _ => {}
        }
    }
    Ok(true)
}
