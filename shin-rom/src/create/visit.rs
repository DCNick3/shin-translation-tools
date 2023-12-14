use std::marker::PhantomData;

use camino::Utf8PathBuf;

use crate::{
    create::source::{FileSource, InputDirectory, InputEntry, InputFile},
    progress::RomCounter,
};

#[allow(unused_variables)] // I don't want to prefix these with _, as it makes the IDE-generated impls have those too
pub trait DirVisitor<'bump, S> {
    // NOTE: while this gives you a mutable reference to the `Utf8PathBuf` for performance reasons,
    // you are supposed to leave it unchanged after the call.
    fn visit_file(
        &mut self,
        index: usize,
        name: &'bump str,
        encoded_name: &'bump [u8],
        path_buf: &mut Utf8PathBuf,
        file: &InputFile<S>,
    ) {
    }
    fn visit_directory(
        &mut self,
        index: usize,
        name: &'bump str,
        encoded_name: &'bump [u8],
        path_buf: &mut Utf8PathBuf,
        directory: &InputDirectory<'bump, S>,
    ) {
    }
}

#[allow(unused_variables)] // I don't want to prefix these with _, as it makes the IDE-generated impls have those too
pub trait FsWalker<'bump, S> {
    fn enter_directory(
        &mut self,
        index: usize,
        name: &'bump str,
        encoded_name: &'bump [u8],
        path_buf: &mut Utf8PathBuf,
        directory: &InputDirectory<'bump, S>,
    );

    fn leave_directory(
        &mut self,
        index: usize,
        name: &'bump str,
        encoded_name: &'bump [u8],
        path_buf: &mut Utf8PathBuf,
        directory: &InputDirectory<'bump, S>,
    ) {
    }
}

pub struct DirVisitorAdapter<'bump, S, DV: DirVisitor<'bump, S>> {
    directory_index: usize,
    file_index: usize,
    visit_root: bool,
    visitor: DV,
    phantom: PhantomData<&'bump S>,
}

impl<'bump, S, DV: DirVisitor<'bump, S>> FsWalker<'bump, S> for DirVisitorAdapter<'bump, S, DV> {
    fn enter_directory(
        &mut self,
        _index: usize,
        _name: &'bump str,
        _encoded_name: &'bump [u8],
        path_buf: &mut Utf8PathBuf,
        directory: &InputDirectory<'bump, S>,
    ) {
        // special case: root directory
        if self.visit_root {
            self.visitor
                .visit_directory(self.directory_index, "", &[], path_buf, directory);
            self.directory_index += 1;
            self.visit_root = false;
        }

        visit_directory(
            directory,
            &mut self.file_index,
            &mut self.directory_index,
            path_buf,
            &mut self.visitor,
        );
    }
}

pub fn walk_input_fs<'bump, S, W>(root: &InputDirectory<'bump, S>, mut walker: W) -> W
where
    W: FsWalker<'bump, S>,
{
    fn recur<'bump, S, V>(
        directory: &InputDirectory<'bump, S>,
        directory_index: usize,
        directory_index_ctr: &mut usize,
        name: &'bump str,
        encoded_name: &'bump [u8],
        path_buf: &mut Utf8PathBuf,
        visitor: &mut V,
    ) where
        V: FsWalker<'bump, S>,
    {
        // trace!("{:10} {}", directory_index, path_buf);
        visitor.enter_directory(directory_index, name, encoded_name, path_buf, directory);

        // this index needs to be consistent with the index computed in `visit_directory`
        // note that the order we assign indices to directories and the order we write them to the ROM are different
        // (maybe it should be changed, but then we would have different logic for directory and file indices)

        // 0 "root"
        // 1 ├─ a
        // 3 │  ├─ c
        // 4 │  └─ d
        // 2 └─ b
        // 5    └─ e

        // 0 /
        // --
        // 1 /a
        // 2 /b
        // --
        // 3 /a/c
        // 4 /a/d
        // --
        // 5 /b/e

        let mut subdir_index = *directory_index_ctr;
        *directory_index_ctr += directory
            .0
            .iter()
            .filter(|(_, _, e)| matches!(e, InputEntry::Directory(_)))
            .count();

        // enter the subdirectories
        for (name, encoded_name, entry) in &directory.0 {
            if let InputEntry::Directory(directory) = entry {
                path_buf.push(name);
                recur(
                    directory,
                    subdir_index,
                    directory_index_ctr,
                    name,
                    encoded_name,
                    path_buf,
                    visitor,
                );
                subdir_index += 1;
                path_buf.pop();
            }
        }
        visitor.leave_directory(directory_index, name, encoded_name, path_buf, directory);
    }

    recur(
        root,
        0,
        &mut 1,
        "",
        &[],
        &mut Utf8PathBuf::new(),
        &mut walker,
    );

    walker
}

pub fn visit_directory<'bump, S, V: DirVisitor<'bump, S>>(
    directory: &InputDirectory<'bump, S>,
    file_index: &mut usize,
    directory_index: &mut usize,
    path_buf: &mut Utf8PathBuf,
    visitor: &mut V,
) {
    for (name, encoded_name, entry) in &directory.0 {
        path_buf.push(name);
        match entry {
            InputEntry::Directory(directory) => {
                visitor.visit_directory(*directory_index, name, encoded_name, path_buf, directory);
                *directory_index += 1;
            }
            InputEntry::File(file) => {
                visitor.visit_file(*file_index, name, encoded_name, path_buf, file);
                *file_index += 1;
            }
        }
        path_buf.pop();
    }
}

pub fn visit_input_fs<'bump, S, V: DirVisitor<'bump, S>>(
    root: &InputDirectory<'bump, S>,
    visitor: V,
) -> V {
    walk_input_fs(
        root,
        DirVisitorAdapter {
            directory_index: 0,
            file_index: 0,
            visit_root: true,
            visitor,
            phantom: PhantomData,
        },
    )
    .visitor
}

impl<'bump, S: FileSource> DirVisitor<'bump, S> for RomCounter {
    fn visit_file(
        &mut self,
        _index: usize,
        _name: &'bump str,
        _encoded_name: &'bump [u8],
        path_buf: &mut Utf8PathBuf,
        file: &InputFile<S>,
    ) {
        self.add_file(
            file.0
                .size(path_buf.as_str())
                .expect("Failed to get file size"),
        );
    }
}
