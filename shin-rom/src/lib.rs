//! A library to create and extract .rom files used by shin-based games.
//!
//! As of now, it only provides very high-level APIs to create a .rom file from a directory and to extract a .rom file to a directory.

// APIs allowing more control over the process would be nice to have.

#[cfg(not(target_pointer_width = "64"))]
// this limitation is due to the use of `usize` for offsets
// and memory-mapping the entire rom, which can be larger than 2GB
compile_error!("shin-rom only supports 64-bit targets");

mod header;
mod progress;

mod create;
mod extract;
mod index;

pub use create::rom_create;
pub use extract::rom_extract;
