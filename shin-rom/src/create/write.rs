use std::io;

use binrw::{io::NoSeek, BinResult, BinWrite};
use bumpalo::{collections, Bump};
use camino::Utf8PathBuf;
use shin_versions::{RomDirectoryOffsetDisposition, RomVersion};
use tracing::trace;

use crate::{
    create::{
        allocate::{AllocatedRom, Allocator},
        source::{FileSource, InputDirectory, InputFile},
        visit,
        visit::{DirVisitor, FsWalker},
    },
    default_spinner_span,
    index::{NameOffsetAndFlags, RawEntry, DIRECTORY_OFFSET_MULTIPLIER},
    progress::{RomCounter, RomProgress},
};

/// A writer that tracks how many bytes were written to it
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
    /// Make sure the current offset is a multiple of `alignment` by padding with zeroes if necessary
    pub fn align(&mut self, alignment: u64) -> io::Result<()> {
        use std::io::Write;

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
    names: collections::Vec<'scratch, &'bump [u8]>,
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

        let names = collections::Vec::with_capacity_in(entry_count, &self.scratch_bump);

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

        visit::visit_directory(
            directory,
            &mut self.file_index,
            &mut self.directory_index,
            &mut Utf8PathBuf::new(),
            &mut visitor,
        );

        // emit the names
        for name in visitor.names {
            use std::io::Write;

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
    use std::io::Write;

    use crate::header::{RomHeader, RomHeaderV1, RomHeaderV2};

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
        visit::walk_input_fs(
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

    let total_count = visit::visit_input_fs(input, RomCounter::new());
    {
        let _span = default_spinner_span!("Writing file contents");
        let mut progress = RomProgress::new(total_count);
        // write all the file contents
        visit::visit_input_fs(
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
