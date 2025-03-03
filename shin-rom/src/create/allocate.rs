use bumpalo::{collections, Bump};
use camino::Utf8PathBuf;
use itertools::Itertools;
use shin_versions::RomVersion;
use tracing::trace;

use crate::{
    create::{
        source::{FileSource, InputDirectory, InputFile},
        visit,
        visit::{DirVisitor, FsWalker},
    },
    header::{RomHeader, RomHeaderV1, RomHeaderV2},
    index::{RawEntry, DIRECTORY_OFFSET_MULTIPLIER},
};

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

pub struct Allocator {
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
    directory_positions: collections::Vec<'bump, (u64, u64)>,
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
                .all(|(a, b)| a.encoded_name < b.encoded_name),
            "directory entries must be sorted"
        );

        let alloc = &mut self.allocator;

        alloc.align(DIRECTORY_OFFSET_MULTIPLIER as u64);

        let entries_count = directory.0.len() + 2; // 2 for "." and ".."
        let entries_size = entries_count * RawEntry::SIZE;

        let my_offset = alloc.allocate(4 + entries_size as u64);

        alloc.allocate(2); // "." entry file name
        alloc.allocate(3); // ".." entry file name

        for entry in &directory.0 {
            alloc.allocate(entry.encoded_name.len() as u64);
        }

        let my_size = alloc.position - my_offset;

        self.directory_positions[index] = (my_offset, my_size);
    }
}

struct AllocateFileVisitor<'a, 'bump> {
    allocator: &'a mut Allocator,
    alignment: u64,
    file_positions: collections::Vec<'bump, (u64, u64)>,
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
    directory_parent_indices: &'a mut collections::Vec<'bump, usize>,
}

struct GatherEntryParents<'bump> {
    directory_index: usize,
    file_index: usize,
    directory_parent_indices: collections::Vec<'bump, usize>,
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

        visit::visit_directory(
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
///
/// This allows to actually write the ROM in one pass, without having to seek back and forth.
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
    } = visit::visit_input_fs(input, CountVisitor::default());

    let index_offset = allocator.position;

    let directory_positions = visit::walk_input_fs(
        input,
        AllocateDirectoryVisitor {
            allocator: &mut allocator,
            directory_positions: collections::Vec::from_iter_in(
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

    let file_positions = visit::visit_input_fs(
        input,
        AllocateFileVisitor {
            allocator: &mut allocator,
            alignment: file_offset_multiplier,
            file_positions: collections::Vec::with_capacity_in(file_count, bump),
        },
    )
    .file_positions;

    allocator.align(file_offset_multiplier);

    assert_eq!(file_positions.len(), file_count);

    let GatherEntryParents {
        directory_parent_indices,
        ..
    } = visit::walk_input_fs(
        input,
        GatherEntryParents {
            directory_index: 0,
            file_index: 0,
            directory_parent_indices: collections::Vec::with_capacity_in(directory_count, bump),
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
