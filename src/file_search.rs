use crate::sort::mergesort;
use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::LinkedList;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use std::{fs, io};

pub struct SearchArgs<'c> {
    pub folder: &'c PathBuf,
    pub find: Option<&'c PathBuf>,
    pub sort: bool,
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
    SearchArgs { folder, find, sort }: SearchArgs,
) -> Result<Vec<SmartPath>, io::Error> {
    let mut ans: LinkedList<SmartPath> = LinkedList::new();

    let files = fs::read_dir(folder)?;
    let filename = find.as_ref().and_then(|p| p.file_name());
    for file in files {
        let file = file?;
        let exists = match filename {
            Some(name) if name == file.file_name() => {
                ans.push_back(SmartPath::new(PathBuf::from(name)));
                true
            }
            None => {
                ans.push_back(SmartPath::new(PathBuf::from(file.file_name())));
                true
            }
            _ => false,
        };

        if file.file_type()?.is_dir() {
            let recursive = file_search(SearchArgs {
                folder: &file.path(),
                find,
                sort,
            })?;
            if recursive.len() > 0 {
                let path = if exists {
                    ans.back_mut().unwrap()
                } else {
                    ans.push_back(SmartPath::new(PathBuf::from(file.file_name())));
                    ans.back_mut().unwrap()
                };
                path.children.replace(recursive);
            }
        }
    }

    let mut ans: Vec<SmartPath> = ans.into_iter().collect();
    if sort {
        mergesort(&mut ans);
    }
    Ok(ans)
}
