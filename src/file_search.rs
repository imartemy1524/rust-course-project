use crate::sort::mergesort;
use log::debug;
use num_cpus;
use std::cmp::Ordering;
use std::io::{Error, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, LazyLock};
use std::thread::spawn;
use std::{fs, io, thread};

pub struct SearchArgs<'c> {
    pub folder: &'c PathBuf,
    pub find: Option<&'c PathBuf>,
    pub sort: bool,
    pub in_file: Option<&'c String>,
}

enum Occurrence {
    File(Arc<PathBuf>),
    Directory(Arc<PathBuf>),
    TextFile(Arc<PathBuf>),
}

pub struct SmartPath {
    fname: Rc<PathBuf>,
    // children: Rc<Cell<Vec<SmartPath>>>,
}
impl SmartPath {
    fn new(string: PathBuf) -> SmartPath {
        SmartPath {
            fname: Rc::new(string),
            // children: Rc::new(Cell::new(Vec::with_capacity(0))),
        }
    }
    pub(crate) fn print(&self, padding: usize, file: &mut Box<dyn Write>) -> io::Result<()> {
        // let items = self.children.take();
        let name = self.fname.to_string_lossy();
        writeln!(file, "{}{} ", " ".repeat(padding), name)?;

        Ok(())
    }
}

impl Clone for SmartPath {
    fn clone(&self) -> Self {
        SmartPath {
            fname: self.fname.clone(),
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

static CURRENT_THREADS_COUNT: AtomicU32 = AtomicU32::new(0);
static THREADS_MAX_COUNT: LazyLock<u32> = LazyLock::new(|| num_cpus::get() as u32);
#[inline]
fn increase_threads_count() -> bool {
    loop {
        let old = CURRENT_THREADS_COUNT.load(core::sync::atomic::Ordering::Acquire);
        if old < *THREADS_MAX_COUNT {

            let new = old + 1;

            let ans = CURRENT_THREADS_COUNT.compare_exchange(
                old,
                new,
                core::sync::atomic::Ordering::AcqRel,
                core::sync::atomic::Ordering::Relaxed,
            );
            if let Err(_) = ans {
                // atomic error, continue
                continue;
            }
            return true;
        }
        return false;
    }
}
struct SmartStructHelperDecreaseCurrentThreadsCount{}
impl Drop for SmartStructHelperDecreaseCurrentThreadsCount {
    fn drop(&mut self) {
        CURRENT_THREADS_COUNT.fetch_sub(1, core::sync::atomic::Ordering::AcqRel);
    }
}
fn search_in_threads<'c>(
    folder: &PathBuf,
    // for some reason when I pass &Sender, the code runs forever, idk why
    // it's possible to pass &Arc<Sender> and Arc<Sender> and the code start working
    sender: Sender<Occurrence>,
) -> Result<(), Error> {
    // thread::scope(move |scope: &Scope| {
    let files = fs::read_dir(folder)?;
    for file in files.into_iter() {
        let file = file?;
        let value = if file.file_type()?.is_dir() {
            let file_clone = file.path().clone();
            // trying to use pseudo-semaphore
            if increase_threads_count() {
                let clone = sender.clone();
                spawn(move || {
                    let q = SmartStructHelperDecreaseCurrentThreadsCount{};
                    let path = file.path().clone();
                    search_in_threads(&path, clone).expect("search_in_threads failed");
                    // decrease the atomic value even if thread panics
                    drop(q);
                });
            } else {
                // do it synchronously
                let path = file.path().clone();
                search_in_threads(&path, sender.clone()).expect("search_in_threads failed");
            }

            Occurrence::Directory(Arc::new(file_clone))
        }
        else if file.file_name().to_string_lossy().ends_with(".txt")
            || file.file_name().to_string_lossy().ends_with(".rs")
        {
            Occurrence::TextFile(Arc::new(file.path()))
        } else {
            Occurrence::File(Arc::new(file.path()))
        };

        if let Err(err) = sender.send(value) {
            return Err(io::Error::new(io::ErrorKind::Other, err.to_string()));
        };
    }

    // Ok(())
    // })?;
    Ok(())
}
pub fn file_search(
    SearchArgs {
        folder,
        find,
        sort,
        in_file,
    }: SearchArgs,
) -> Result<Vec<SmartPath>, io::Error> {
    // LinkedList::new();

    let (sender, receiver) = channel();
    // let ans = thread::scope(|scope: Scope| {
    search_in_threads(folder, sender)?;
    // });
    let ans = receiver.iter();

    let mut answer: Vec<SmartPath> = Vec::new();
    for i in ans {
        if should_include(&i, &in_file, &find)? {
            match i {
                Occurrence::File(i) | Occurrence::TextFile(i) => {
                    let path = (*i).clone();
                    answer.push(SmartPath::new(path));
                }
                Occurrence::Directory(i) => {
                    let q = (*i).clone();
                    answer.push(SmartPath::new(q));
                }
            };
        }
    }
    if sort {
        mergesort(&mut answer);
    }
    Ok(answer.iter().map(|e| e.clone()).collect())
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
        } else {
            return Ok(false);
        }
    }
    if let Some(find) = find.as_ref().and_then(|p| p.file_name()) {
        let name = match item {
            Occurrence::File(q) | Occurrence::TextFile(q) | Occurrence::Directory(q) => {
                q.as_ref().file_name()
            }
        };
        match name {
            None => return Ok(false),
            Some(name) if name != find => return Ok(false),
            _ => {}
        }
    }
    Ok(true)
}
