use std::io;

use bumpalo::{collections, Bump};
use camino::{Utf8Path, Utf8PathBuf};
use shin_text::encode_sjis_string;
use shin_versions::RomEncoding;
use tracing::warn;

pub enum InputEntry<'bump, S> {
    Directory(InputDirectory<'bump, S>),
    File(InputFile<S>),
}

pub struct InputDirectory<'bump, S>(
    pub collections::Vec<'bump, (&'bump str, &'bump [u8], InputEntry<'bump, S>)>,
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
            let mut result = collections::Vec::new_in(bump);

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

                let name = collections::String::from_str_in(file_name, bump).into_bump_str();
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
