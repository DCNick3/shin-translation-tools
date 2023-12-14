use std::{io, marker::PhantomData};

use binrw::{io::NoSeek, BinResult, BinWrite as _};
use bumpalo::{
    collections::{String, Vec},
    Bump,
};
use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools as _;
use shin_text::encode_sjis_string;
use shin_versions::{RomDirectoryOffsetDisposition, RomEncoding, RomVersion};
use tracing::{trace, warn};

use crate::{
    default_spinner_span,
    header::{RomHeader, RomHeaderV1, RomHeaderV2},
    index::{NameOffsetAndFlags, RawEntry, DIRECTORY_OFFSET_MULTIPLIER},
    progress::{RomCounter, RomProgress},
};

#[allow(unused_variables)] // I don't want to prefix these with _, as it makes the IDE-generated impls have those too
trait DirVisitor<'bump, S> {
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
trait FsWalker<'bump, S> {
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

struct DirVisitorAdapter<'bump, S, DV: DirVisitor<'bump, S>> {
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

fn walk_input_fs<'bump, S, W>(root: &InputDirectory<'bump, S>, mut walker: W) -> W
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

fn visit_directory<'bump, S, V: DirVisitor<'bump, S>>(
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

fn visit_input_fs<'bump, S, V: DirVisitor<'bump, S>>(
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

pub enum InputEntry<'bump, S> {
    Directory(InputDirectory<'bump, S>),
    File(InputFile<S>),
}

pub struct InputDirectory<'bump, S>(
    pub Vec<'bump, (&'bump str, &'bump [u8], InputEntry<'bump, S>)>,
);

impl<'bump, 'a> InputDirectory<'bump, BaseDirFileSource<'a>> {
    pub fn walk(bump: &'bump Bump, encoding: RomEncoding, base_dir: &'a Utf8Path) -> Self {
        fn recur<'bump, 'a>(
            bump: &'bump Bump,
            encoding: RomEncoding,
            base_dir: &'a Utf8Path,
            path_buf: &mut Utf8PathBuf,
        ) -> InputDirectory<'bump, BaseDirFileSource<'a>> {
            // TODO: know capacity beforehand?
            let mut result = Vec::new_in(bump);

            for v in std::fs::read_dir(&path_buf).expect("Failed reading directory for rom") {
                let v = v.expect("Failed reading directory for rom");
                let ty = v.file_type().expect("Failed to get file type for rom");
                if !ty.is_dir() && !ty.is_file() {
                    // TODO: resolve symlinks?
                    warn!("Skipping non-file, non-directory {:?}", v.path());
                    continue;
                }

                let file_name = v.file_name();
                let file_name = file_name.to_str().expect("invalid utf8 in rom file");

                let name = String::from_str_in(file_name, bump).into_bump_str();
                let encoded_name = match encoding {
                    RomEncoding::Utf8 => name.as_bytes(),
                    RomEncoding::ShiftJIS => encode_sjis_string(bump, name, false)
                        .expect("filename not encodable in Shift-JIS")
                        .into_bump_slice(),
                };
                let entry = if ty.is_dir() {
                    InputEntry::Directory({
                        path_buf.push(file_name);

                        let dir = recur(bump, encoding, base_dir, path_buf);

                        path_buf.pop();

                        dir
                    })
                } else if ty.is_file() {
                    InputEntry::File(InputFile(BaseDirFileSource { base_dir }))
                } else {
                    unreachable!()
                };

                result.push((name, encoded_name, entry))
            }

            result.sort_by(|(_, a, _), (_, b, _)| a.cmp(b));

            InputDirectory(result)
        }

        let mut s = base_dir.to_path_buf();
        recur(bump, encoding, base_dir, &mut s)
    }
}

pub struct InputFile<S>(pub S);

pub trait FileSource {
    type Stream: io::Read;

    fn open(&self, path: &str) -> io::Result<Self::Stream>;
    fn size(&self, path: &str) -> io::Result<u64>;
}

pub struct BaseDirFileSource<'a> {
    pub base_dir: &'a Utf8Path,
}

impl<'a> FileSource for BaseDirFileSource<'a> {
    type Stream = std::fs::File;

    fn open(&self, path: &str) -> io::Result<Self::Stream> {
        let path = self.base_dir.join(path);
        std::fs::File::open(path)
    }

    fn size(&self, path: &str) -> io::Result<u64> {
        let path = self.base_dir.join(path);
        std::fs::metadata(path).map(|m| m.len())
    }
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

#[derive(Default)]
struct CountVisitor {
    pub directory_count: usize,
    pub file_count: usize,
}

impl<'bump, S> DirVisitor<'bump, S> for CountVisitor {
    fn visit_file(
        &mut self,
        _file_index: usize,
        _name: &'bump str,
        _encoded_name: &'bump [u8],
        _path: &mut Utf8PathBuf,
        _file: &InputFile<S>,
    ) {
        self.file_count += 1;
    }

    fn visit_directory(
        &mut self,
        _directory_index: usize,
        _name: &'bump str,
        _encoded_name: &'bump [u8],
        _path: &mut Utf8PathBuf,
        _directory: &InputDirectory<'bump, S>,
    ) {
        self.directory_count += 1;
    }
}

struct Allocator {
    pub position: u64,
}

impl Allocator {
    pub fn new(initial_position: u64) -> Self {
        Allocator {
            position: initial_position,
        }
    }

    pub fn allocate(&mut self, size: u64) -> u64 {
        let position = self.position;
        self.position += size;
        position
    }

    pub fn align(&mut self, alignment: u64) {
        self.position = self.position.next_multiple_of(alignment);
    }
}

struct AllocateDirectoryVisitor<'a, 'bump> {
    allocator: &'a mut Allocator,
    directory_positions: Vec<'bump, (u64, u64)>,
}

impl<'a, 'bump, S> FsWalker<'bump, S> for AllocateDirectoryVisitor<'a, 'bump> {
    fn enter_directory(
        &mut self,
        index: usize,
        _name: &'bump str,
        _encoded_name: &'bump [u8],
        _path_buf: &mut Utf8PathBuf,
        directory: &InputDirectory<'bump, S>,
    ) {
        // NOTE: this is WRONG
        // the directories must be sorted in the ROM encoding, not in UTF-8
        assert!(
            directory
                .0
                .iter()
                .tuple_windows()
                .all(|((_, a, _), (_, b, _))| a < b),
            "directory entries must be sorted"
        );

        let alloc = &mut self.allocator;

        alloc.align(DIRECTORY_OFFSET_MULTIPLIER as u64);

        let entries_count = directory.0.len() + 2; // 2 for "." and ".."
        let entries_size = entries_count * RawEntry::SIZE;

        let my_offset = alloc.allocate(4 + entries_size as u64);

        alloc.allocate(2); // "." entry file name
        alloc.allocate(3); // ".." entry file name

        for (_, encoded_name, _) in &directory.0 {
            alloc.allocate(encoded_name.len() as u64 + 1);
        }

        let my_size = alloc.position - my_offset;

        self.directory_positions[index] = (my_offset, my_size);
    }
}

struct AllocateFileVisitor<'a, 'bump> {
    allocator: &'a mut Allocator,
    alignment: u64,
    file_positions: Vec<'bump, (u64, u64)>,
}

impl<'a, 'bump, S: FileSource> DirVisitor<'bump, S> for AllocateFileVisitor<'a, 'bump> {
    fn visit_file(
        &mut self,
        file_index: usize,
        _name: &'bump str,
        _encoded_name: &'bump [u8],
        path: &mut Utf8PathBuf,
        file: &InputFile<S>,
    ) {
        self.allocator.align(self.alignment);
        let my_offset = self.allocator.position;
        let my_size = file.0.size(path.as_str()).expect("Failed to get file size");

        trace!(
            "{my_offset:#018x} {:#010x} {my_size:#010x} {path:80}",
            my_offset / 0x800
        );

        self.allocator.allocate(my_size);

        debug_assert_eq!(self.file_positions.len(), file_index);
        self.file_positions.push((my_offset, my_size));
    }
}

struct GatherEntryParentsInnerVisitor<'a, 'bump> {
    parent_index: usize,
    directory_parent_indices: &'a mut Vec<'bump, usize>,
}

struct GatherEntryParents<'bump> {
    directory_index: usize,
    file_index: usize,
    directory_parent_indices: Vec<'bump, usize>,
}

impl<'a, 'bump, S> DirVisitor<'bump, S> for GatherEntryParentsInnerVisitor<'a, 'bump> {
    fn visit_directory(
        &mut self,
        _directory_index: usize,
        _name: &'bump str,
        _encoded_name: &'bump [u8],
        _path: &mut Utf8PathBuf,
        _directory: &InputDirectory<'bump, S>,
    ) {
        self.directory_parent_indices.push(self.parent_index);
    }
}

impl<'bump, S> FsWalker<'bump, S> for GatherEntryParents<'bump> {
    fn enter_directory(
        &mut self,
        index: usize,
        name: &'bump str,
        encoded_name: &'bump [u8],
        path_buf: &mut Utf8PathBuf,
        directory: &InputDirectory<'bump, S>,
    ) {
        if index == 0 {
            // this is the root directory, needs special care
            let mut visitor = GatherEntryParentsInnerVisitor {
                parent_index: 0,
                directory_parent_indices: &mut self.directory_parent_indices,
            };
            assert_eq!(index, 0);
            visitor.visit_directory(0, name, encoded_name, path_buf, directory);
            self.directory_index += 1;
        }

        // trace!("{:10} {}", index, path_buf);

        let mut visitor = GatherEntryParentsInnerVisitor {
            parent_index: index,
            directory_parent_indices: &mut self.directory_parent_indices,
        };

        visit_directory(
            directory,
            &mut self.file_index,
            &mut self.directory_index,
            path_buf,
            &mut visitor,
        );
    }
}

pub struct AllocatedRom<'bump> {
    pub directory_positions: &'bump [(u64, u64)],
    pub file_positions: &'bump [(u64, u64)],
    pub directory_parent_indices: &'bump [usize],
    pub index_offset: u64,
    pub index_size: u64,
    pub file_size: u64,
    pub file_offset_multiplier: u64,
}

/// Generates the full layout of the ROM, planning on where to place each entry and data.
pub fn rom_allocate<'bump, S: FileSource>(
    bump: &'bump Bump,
    rom_version: RomVersion,
    input: &InputDirectory<'bump, S>,
) -> AllocatedRom<'bump> {
    let mut allocator = Allocator::new(0);

    allocator.allocate(RomVersion::HEAD_BYTES_SIZE as u64);
    allocator.allocate(match rom_version {
        RomVersion::Rom1V2_1 => RomHeaderV1::SIZE,
        RomVersion::Rom2V1_0 | RomVersion::Rom2V1_1 => RomHeaderV2::SIZE,
    } as u64);

    let CountVisitor {
        directory_count,
        file_count,
    } = visit_input_fs(input, CountVisitor::default());

    let index_offset = allocator.position;

    let directory_positions = walk_input_fs(
        input,
        AllocateDirectoryVisitor {
            allocator: &mut allocator,
            directory_positions: Vec::from_iter_in(
                std::iter::repeat((u64::MAX, u64::MAX)).take(directory_count),
                bump,
            ),
        },
    )
    .directory_positions;

    assert_eq!(directory_positions.len(), directory_count);

    // do a one last alignment before the file data to aligh the index_size
    allocator.align(16);

    let index_size = allocator.position - index_offset;
    let file_offset_multiplier = RomHeader::default_file_offset_multiplier(rom_version) as u64;

    let file_positions = visit_input_fs(
        input,
        AllocateFileVisitor {
            allocator: &mut allocator,
            alignment: file_offset_multiplier,
            file_positions: Vec::with_capacity_in(file_count, bump),
        },
    )
    .file_positions;

    allocator.align(file_offset_multiplier);

    assert_eq!(file_positions.len(), file_count);

    let GatherEntryParents {
        directory_parent_indices,
        ..
    } = walk_input_fs(
        input,
        GatherEntryParents {
            directory_index: 0,
            file_index: 0,
            directory_parent_indices: Vec::with_capacity_in(directory_count, bump),
        },
    );

    assert_eq!(directory_parent_indices.len(), directory_count);

    AllocatedRom {
        directory_positions: directory_positions.into_bump_slice(),
        file_positions: file_positions.into_bump_slice(),
        directory_parent_indices: directory_parent_indices.into_bump_slice(),
        index_offset,
        index_size,
        file_size: allocator.position,
        file_offset_multiplier,
    }
}

struct WriteWrapper<W> {
    writer: W,
    offset: u64,
}

impl<W: io::Write> io::Write for WriteWrapper<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.writer.write(buf)?;
        self.offset += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: io::Write> WriteWrapper<W> {
    pub fn align(&mut self, alignment: u64) -> io::Result<()> {
        use io::Write;

        let delta = self.offset.next_multiple_of(alignment) - self.offset;
        for _ in 0..delta {
            self.write_all(&[0])?;
        }

        Ok(())
    }
}

struct WriteDirectoryInnerVisitor<'scratch, 'a, 'bump, W> {
    directory_offset_disposition: RomDirectoryOffsetDisposition,

    index_offset: u64,
    file_offset_multiplier: u64,
    directory_positions: &'bump [(u64, u64)],
    file_positions: &'bump [(u64, u64)],
    writer: &'a mut WriteWrapper<W>,
    names_allocator: Allocator,
    names: Vec<'scratch, &'bump [u8]>,
}
impl<'scratch, 'a, 'bump, W: io::Write> WriteDirectoryInnerVisitor<'scratch, 'a, 'bump, W> {
    fn emit_entry(
        &mut self,
        name: &'bump str,
        encoded_name: &'bump [u8],
        is_directory: bool,
        (mut offset, size): (u64, u64),
    ) {
        self.names.push(encoded_name);

        let name_size = encoded_name.len() as u64 + 1;
        let name_offset = self.names_allocator.allocate(name_size);

        let offset_multiplier = if is_directory {
            DIRECTORY_OFFSET_MULTIPLIER as u64
        } else {
            self.file_offset_multiplier
        };

        if is_directory {
            match self.directory_offset_disposition {
                RomDirectoryOffsetDisposition::FromStart => {}
                RomDirectoryOffsetDisposition::FromIndexStart => {
                    offset -= self.index_offset;
                }
            }
        }

        let name_and_flags = NameOffsetAndFlags(0)
            .with_is_directory(is_directory)
            .with_name_offset(name_offset.try_into().unwrap());
        let data_offset = (offset / offset_multiplier)
            .try_into()
            .expect("rom offset too large");
        let data_size = size.try_into().expect("file too large");

        trace!(
            "{offset:#018x} {:#010x} {data_offset:#010x} {data_size:#010x} {name:24}",
            name_and_flags.0
        );

        let entry = RawEntry {
            name_and_flags,
            data_offset,
            data_size,
        };
        entry
            .write_le(&mut NoSeek::new(&mut self.writer))
            .expect("Failed to write entry");
    }
}

impl<'scratch, 'a, 'bump, W: io::Write, S> DirVisitor<'bump, S>
    for WriteDirectoryInnerVisitor<'scratch, 'a, 'bump, W>
{
    fn visit_file(
        &mut self,
        index: usize,
        name: &'bump str,
        encoded_name: &'bump [u8],
        _path_buf: &mut Utf8PathBuf,
        _file: &InputFile<S>,
    ) {
        let file_position = self.file_positions[index];
        self.emit_entry(name, encoded_name, false, file_position);
    }

    fn visit_directory(
        &mut self,
        index: usize,
        name: &'bump str,
        encoded_name: &'bump [u8],
        _path_buf: &mut Utf8PathBuf,
        _directory: &InputDirectory<'bump, S>,
    ) {
        let directory_position = self.directory_positions[index];
        self.emit_entry(name, encoded_name, true, directory_position);
    }
}

struct WriteDirectoryWalker<'a, 'bump, W> {
    scratch_bump: Bump,

    directory_offset_disposition: RomDirectoryOffsetDisposition,

    index_offset: u64,
    file_offset_multiplier: u64,
    directory_positions: &'bump [(u64, u64)],
    file_positions: &'bump [(u64, u64)],
    directory_parent_indices: &'bump [usize],
    // for the inner visitor
    directory_index: usize,
    file_index: usize,

    writer: &'a mut WriteWrapper<W>,
}

impl<'a, 'bump, W, S> FsWalker<'bump, S> for WriteDirectoryWalker<'a, 'bump, W>
where
    W: io::Write,
{
    fn enter_directory(
        &mut self,
        index: usize,
        _name: &'bump str,
        _encoded_name: &'bump [u8],
        _path: &mut Utf8PathBuf,
        directory: &InputDirectory<'bump, S>,
    ) {
        self.scratch_bump.reset();

        let current_directory_position = self.directory_positions[index];

        self.writer
            .align(DIRECTORY_OFFSET_MULTIPLIER as u64)
            .expect("Failed to align");

        assert_eq!(current_directory_position.0, self.writer.offset);

        let entry_count = directory.0.len() + 2; // 2 for "." and ".."

        (entry_count as u32)
            .write_le(&mut NoSeek::new(&mut self.writer))
            .expect("Failed to write entry length");

        // the directory first lists all its entries (in a fixed-size records), then all the names referred in the entries
        // we do not accumulate anything unnecessary in the memory, so we just track our offsets to the names with `names_allocator`
        // this brings a lot of potential for bugs (if the actually emitted names do not match the allocated offsets)
        // but it's also very fast and memory-efficient
        let names_allocator = Allocator::new((4 + entry_count * RawEntry::SIZE) as u64);

        let names = Vec::with_capacity_in(entry_count, &self.scratch_bump);

        let mut visitor = WriteDirectoryInnerVisitor {
            directory_offset_disposition: self.directory_offset_disposition,

            index_offset: self.index_offset,
            file_offset_multiplier: self.file_offset_multiplier,
            directory_positions: self.directory_positions,
            file_positions: self.file_positions,
            writer: self.writer,
            names_allocator,
            names,
        };

        // emit "." and ".." entries (they are always at the beginning)
        visitor.emit_entry(".", b".", true, current_directory_position);
        let parent_index = self.directory_parent_indices[index];
        let parent_directory_position = self.directory_positions[parent_index];
        visitor.emit_entry("..", b"..", true, parent_directory_position);

        visit_directory(
            directory,
            &mut self.file_index,
            &mut self.directory_index,
            &mut Utf8PathBuf::new(),
            &mut visitor,
        );

        // emit the names
        for name in visitor.names {
            use io::Write;

            self.writer.write_all(name).expect("Failed to write name");
            self.writer.write_all(&[0]).expect("Failed to write name");
        }
    }
}

struct WriteFileVisitor<'a, 'bump, W> {
    file_offset_multiplier: u64,
    file_positions: &'bump [(u64, u64)],
    writer: &'a mut WriteWrapper<W>,
    progress: &'a mut RomProgress,
}

impl<'a, 'bump, W: io::Write, S: FileSource> DirVisitor<'bump, S>
    for WriteFileVisitor<'a, 'bump, W>
{
    fn visit_file(
        &mut self,
        index: usize,
        _name: &'bump str,
        _encoded_name: &'bump [u8],
        path_buf: &mut Utf8PathBuf,
        file: &InputFile<S>,
    ) {
        let (offset, size) = self.file_positions[index];
        self.writer.align(self.file_offset_multiplier).unwrap();

        assert_eq!(offset, self.writer.offset);

        let mut stream = file
            .0
            .open(path_buf.as_str())
            .unwrap_or_else(|e| panic!("Failed to open file {:?}: {:?}", path_buf, e));
        std::io::copy(&mut stream, &mut self.writer)
            .unwrap_or_else(|e| panic!("Failed to copy file {:?} to rom: {:?}", path_buf, e));

        self.progress.add_file(size);

        assert_eq!(
            size,
            self.writer.offset - offset,
            "File size mismatch for {:?}, did it change during the rom build?",
            path_buf
        );
    }
}

pub fn rom_write<'bump, S: FileSource, W: io::Write>(
    rom_version: RomVersion,
    input: &InputDirectory<'bump, S>,
    allocated: &AllocatedRom<'bump>,
    writer: &mut W,
) -> BinResult<RomCounter> {
    use io::Write;

    let _span = default_spinner_span!("Writing rom contents");

    let mut writer = WriteWrapper { writer, offset: 0 };

    writer.write_all(&rom_version.head_bytes())?;

    match rom_version {
        RomVersion::Rom1V2_1 => {
            assert_eq!(
                allocated.file_offset_multiplier,
                RomHeaderV1::DEFAULT_FILE_OFFSET_MULTIPLIER as u64
            );
            RomHeader::V1(RomHeaderV1 {
                index_size: allocated.index_size.try_into().unwrap(),
                // these bytes appear to be random (hash of the data?)
                // I don't know the algo and games doesn't use it
                // so put bytes attributed to our tool here
                unk: *b"Shin",
            })
        }
        RomVersion::Rom2V1_0 | RomVersion::Rom2V1_1 => RomHeader::V2(RomHeaderV2 {
            index_size: allocated.index_size.try_into().unwrap(),
            file_offset_multiplier: allocated.file_offset_multiplier.try_into().unwrap(),
            // these bytes appear to be random (hash of the data?)
            // I don't know the algo and games doesn't use it
            // so put bytes attributed to our tool here
            unk: *b"ShinTransltTools",
        }),
    }
    .write_le(&mut NoSeek::new(&mut writer))?;

    // NOTE: we don't handle io errors in the fs visitor because it's too complicated (at least for now...)
    // ideally we would use a lending iterator instead of a visitor, but I couldn't get it to work:
    // https://github.com/Crazytieguy/gat-lending-iterator/issues/20

    // write all the directory indices
    {
        let _span = default_spinner_span!("Writing directory indices");
        walk_input_fs(
            input,
            WriteDirectoryWalker {
                scratch_bump: Bump::new(),

                directory_offset_disposition: rom_version.directory_offset_disposition(),

                index_offset: allocated.index_offset,
                file_offset_multiplier: RomHeader::default_file_offset_multiplier(rom_version)
                    as u64,
                directory_positions: allocated.directory_positions,
                file_positions: allocated.file_positions,
                directory_parent_indices: allocated.directory_parent_indices,
                directory_index: 1, // to compensate for the root directory
                file_index: 0,
                writer: &mut writer,
            },
        );
    }

    let total_count = visit_input_fs(input, RomCounter::new());
    {
        let _span = default_spinner_span!("Writing file contents");
        let mut progress = RomProgress::new(total_count);
        // write all the file contents
        visit_input_fs(
            input,
            WriteFileVisitor {
                file_offset_multiplier: allocated.file_offset_multiplier,
                file_positions: allocated.file_positions,
                writer: &mut writer,
                progress: &mut progress,
            },
        );
    }

    // align the end-of-file
    writer.align(allocated.file_offset_multiplier)?;
    writer.flush()?;

    Ok(total_count)
}
