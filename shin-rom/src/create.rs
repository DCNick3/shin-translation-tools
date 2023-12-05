use bumpalo::{
    collections::{String, Vec},
    Bump,
};
use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools as _;
use shin_text::{encode_sjis_string, measure_sjis_zstring, Cow};
use shin_versions::{RomEncoding, RomVersion};

use crate::{
    header::{RomHeader, RomHeaderV1, RomHeaderV2},
    index::{RawEntry, DIRECTORY_OFFSET_MULTIPLIER},
};

enum DirectoryAction {
    Enter,
    // maybe I'll use it later, idk
    #[allow(unused)]
    Skip,
}

trait FsVisitor<F, D> {
    fn visit_file(&mut self, path: &str, file: &F);
    fn visit_directory(&mut self, path: &str, directory: &D) -> DirectoryAction;
}

fn visit_input_fs<'bump, S, V>(root: &InputDirectory<'bump, S>, mut visitor: V) -> V
where
    V: FsVisitor<InputFile<S>, InputDirectory<'bump, S>>,
{
    fn recur<'bump, S, V>(
        dir: &InputDirectory<'bump, S>,
        path_buf: &mut Utf8PathBuf,
        visitor: &mut V,
    ) where
        V: FsVisitor<InputFile<S>, InputDirectory<'bump, S>>,
    {
        for (name, entry) in &dir.0 {
            path_buf.push(name.as_str());
            match entry {
                InputEntry::Directory(directory) => {
                    match visitor.visit_directory(path_buf.as_str(), directory) {
                        DirectoryAction::Enter => {
                            recur(directory, path_buf, visitor);
                        }
                        DirectoryAction::Skip => {}
                    }
                }
                InputEntry::File(file) => visitor.visit_file(path_buf.as_str(), file),
            }
            path_buf.pop();
        }
    }

    recur(root, &mut Utf8PathBuf::new(), &mut visitor);

    visitor
}

pub enum InputEntry<'bump, S> {
    Directory(InputDirectory<'bump, S>),
    File(InputFile<S>),
}

pub struct InputDirectory<'bump, S>(pub Vec<'bump, (String<'bump>, InputEntry<'bump, S>)>);

impl<'bump, 'a> InputDirectory<'bump, BaseDirFileSource<'a>> {
    pub fn walk(bump: &'bump Bump, base_dir: &'a Utf8Path) -> Self {
        fn recur<'bump, 'a>(
            bump: &'bump Bump,
            base_dir: &'a Utf8Path,
            path_buf: &mut Utf8PathBuf,
        ) -> InputDirectory<'bump, BaseDirFileSource<'a>> {
            // TODO: know capacity beforehand?
            let mut result = Vec::new_in(bump);

            for v in std::fs::read_dir(&path_buf).expect("Failed reading directory for rom") {
                let v = v.expect("Failed reading directory for rom");
                let ty = v.file_type().expect("Failed to get file type for rom");
                if !ty.is_dir() && !ty.is_file() {
                    eprintln!("Skipping non-file, non-directory {:?}", v.path());
                    continue;
                }

                let file_name = v.file_name();
                let file_name = file_name.to_str().expect("invalid utf8 in rom file");

                result.push((
                    String::from_str_in(file_name, bump),
                    if ty.is_dir() {
                        InputEntry::Directory({
                            path_buf.push(file_name);

                            let dir = recur(bump, base_dir, path_buf);

                            path_buf.pop();

                            dir
                        })
                    } else if ty.is_file() {
                        InputEntry::File(InputFile(BaseDirFileSource { base_dir }))
                    } else {
                        unreachable!()
                    },
                ))
            }

            result.sort_by(|(a, _), (b, _)| a.cmp(b));

            InputDirectory(result)
        }

        let mut s = base_dir.to_path_buf();
        recur(bump, base_dir, &mut s)
    }
}

pub struct InputFile<S>(pub S);

pub trait FileSource {
    type Stream: std::io::Read;

    fn open(&self, path: &str) -> std::io::Result<Self::Stream>;
    fn size(&self, path: &str) -> std::io::Result<u64>;
}

pub struct BaseDirFileSource<'a> {
    pub base_dir: &'a Utf8Path,
}

impl<'a> FileSource for BaseDirFileSource<'a> {
    type Stream = std::fs::File;

    fn open(&self, path: &str) -> std::io::Result<Self::Stream> {
        let path = self.base_dir.join(path);
        std::fs::File::open(path)
    }

    fn size(&self, path: &str) -> std::io::Result<u64> {
        let path = self.base_dir.join(path);
        std::fs::metadata(path).map(|m| m.len())
    }
}

#[derive(Default)]
struct CountVisitor {
    pub directory_count: usize,
    pub file_count: usize,
}

impl<F, D> FsVisitor<F, D> for CountVisitor {
    fn visit_file(&mut self, _path: &str, _file: &F) {
        self.file_count += 1;
    }

    fn visit_directory(&mut self, _path: &str, _directory: &D) -> DirectoryAction {
        self.directory_count += 1;
        DirectoryAction::Enter
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
    rom_encoding: RomEncoding,
    allocator: &'a mut Allocator,
    directory_positions: &'a mut Vec<'bump, (u64, u64)>,
}

impl<'a, 'bump, S> FsVisitor<InputFile<S>, InputDirectory<'bump, S>>
    for AllocateDirectoryVisitor<'a, 'bump>
{
    fn visit_file(&mut self, _path: &str, _file: &InputFile<S>) {}

    fn visit_directory(
        &mut self,
        _path: &str,
        directory: &InputDirectory<'bump, S>,
    ) -> DirectoryAction {
        assert!(
            directory
                .0
                .iter()
                .tuple_windows()
                .all(|((a, _), (b, _))| a < b),
            "directory entries must be sorted"
        );

        let alloc = &mut self.allocator;

        alloc.align(DIRECTORY_OFFSET_MULTIPLIER as u64);

        let entries_count = directory.0.len() + 2; // 2 for "." and ".."
        let entries_size = entries_count * RawEntry::SIZE;

        let my_offset = alloc.allocate(4 + entries_size as u64);

        alloc.allocate(2); // "." entry file name
        alloc.allocate(3); // ".." entry file name

        for (name, _) in &directory.0 {
            let encoded_name_len = match self.rom_encoding {
                RomEncoding::Utf8 => name.len() + 1,
                RomEncoding::ShiftJIS => {
                    measure_sjis_zstring(name).expect("filename not encodable in Shift-JIS")
                }
            };
            alloc.allocate(encoded_name_len as u64);
        }

        let my_size = alloc.position - my_offset;

        self.directory_positions.push((my_offset, my_size));

        DirectoryAction::Enter
    }
}

struct AllocateFileVisitor<'a, 'bump> {
    allocator: &'a mut Allocator,
    alignment: u64,
    file_positions: &'a mut Vec<'bump, (u64, u64)>,
}

impl<'a, 'bump, S: FileSource> FsVisitor<InputFile<S>, InputDirectory<'bump, S>>
    for AllocateFileVisitor<'a, 'bump>
{
    fn visit_file(&mut self, path: &str, file: &InputFile<S>) {
        self.allocator.align(self.alignment);

        let my_offset = self.allocator.position;
        let my_size = file.0.size(path).expect("Failed to get file size");

        self.allocator.allocate(my_size);

        self.file_positions.push((my_offset, my_size));
    }

    fn visit_directory(
        &mut self,
        _path: &str,
        _directory: &InputDirectory<'bump, S>,
    ) -> DirectoryAction {
        DirectoryAction::Enter
    }
}

pub struct AllocatedIndex<'bump> {
    pub directory_positions: Vec<'bump, (u64, u64)>,
    pub file_positions: Vec<'bump, (u64, u64)>,
    pub index_offset: u64,
    pub index_size: u64,
    pub file_size: u64,
}

pub fn allocate_index<'bump, S: FileSource>(
    bump: &'bump Bump,
    rom_version: RomVersion,
    input: &InputDirectory<'bump, S>,
) -> AllocatedIndex<'bump> {
    let mut allocator = Allocator::new(0);

    allocator.allocate(RomVersion::HEAD_BYTES_SIZE as u64);
    allocator.allocate(match rom_version {
        RomVersion::Rom1V2_1 => RomHeaderV1::SIZE,
        RomVersion::Rom2V0_1 | RomVersion::Rom2V1_1 => RomHeaderV2::SIZE,
    } as u64);

    let CountVisitor {
        directory_count,
        file_count,
    } = visit_input_fs(input, CountVisitor::default());
    let mut directory_positions = Vec::with_capacity_in(directory_count, bump);

    let index_offset = allocator.position;

    visit_input_fs(
        input,
        AllocateDirectoryVisitor {
            rom_encoding: rom_version.encoding(),
            allocator: &mut allocator,
            directory_positions: &mut directory_positions,
        },
    );

    assert_eq!(directory_positions.len(), directory_count);

    let index_size = allocator.position - index_offset;
    let file_offset_multiplier = RomHeader::default_file_offset_multiplier(rom_version) as u64;

    let mut file_positions = Vec::with_capacity_in(file_count, bump);

    visit_input_fs(
        input,
        AllocateFileVisitor {
            allocator: &mut allocator,
            alignment: file_offset_multiplier,
            file_positions: &mut file_positions,
        },
    );

    allocator.align(file_offset_multiplier);

    assert_eq!(file_positions.len(), file_count);

    AllocatedIndex {
        directory_positions,
        file_positions,
        index_offset,
        index_size,
        file_size: allocator.position,
    }
}
