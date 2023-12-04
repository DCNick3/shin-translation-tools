#[cfg(not(target_pointer_width = "64"))]
// this limitation is due to the use of `usize` for offsets
// and memory-mapping the entire rom, which can be larger than 2GB
compile_error!("shin-rom only supports 64-bit targets");

mod actions;
mod header;
mod index;
mod progress;
mod version;

use clap::Parser;

use crate::index::{DirectoryIter, DirectoryIterCtx, EntryContent};

#[derive(Parser)]
enum Command {
    Extract(actions::Extract),
}

// this could be made into a proper iterator, but:
// 1. it's tedious to manage all that nested interators
// 2. we wouldn't be able to re-use the path buffer (need a lending iterator for it)
pub fn iter_rom<F: FnMut(&str, &EntryContent)>(ctx: &DirectoryIterCtx, mut f: F) {
    fn recur<F: FnMut(&str, &EntryContent)>(f: &mut F, path_buf: &mut String, iter: DirectoryIter) {
        for entry in iter {
            path_buf.push_str(&entry.name);
            f(&path_buf, &entry.content);
            match entry.content {
                EntryContent::File(_) => {}
                EntryContent::Directory(iter) => {
                    path_buf.push('/');
                    recur(f, path_buf, iter);
                    path_buf.pop().unwrap();
                }
            }
            path_buf.truncate(path_buf.len() - entry.name.len());
        }
    }

    recur(&mut f, &mut String::new(), DirectoryIter::new(&ctx, 0));
}

fn main() {
    match Command::parse() {
        Command::Extract(action) => action.run(),
    }
}
