use flate2::read::ZlibDecoder;
use memmap2::Mmap;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::HashSet,
    fs::{create_dir_all, File},
    io::{self, Write},
    path::Path,
};

use crate::{errors::LibraryError, reader::Reader};

#[derive(Debug, Clone)]
pub struct LibraryRecord {
    offset: u32,
    size: u32,
    zip_size: u32,
    zipped: bool,
    crc32: u32,
    file_name: String,
}

#[derive(Debug)]
pub struct Library {
    version: u32,
    file_count: u32,
    files: Vec<LibraryRecord>,
    mmap: Mmap,
}

const MAGIC_HEADER: &[u8; 5] = b"KIWAD";

impl Library {
    /// Opens and parses a `.wad` file by memory-mapping it instead of
    /// reading it fully into a `Vec<u8>` up front.
    ///
    /// # Errors
    /// Returns an error if the file can't be opened/mapped, or the KIWAD
    /// header/records are malformed.
    pub fn new<P>(path: P) -> Result<Self, LibraryError>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path.as_ref())?;
        let mmap = unsafe { Mmap::map(&file)? };
        #[cfg(unix)]
        {
            use memmap2::Advice;
            mmap.advise(Advice::Sequential)?;
        }

        let mut reader = Reader::new(&mmap);
        let header = reader.take(5)?;

        if Self::is_magic_header(header) {
            let version = reader.read_u32()?;
            let file_count = reader.read_u32()?;

            if version >= 2 {
                reader.take(1)?;
            }

            let mut files: Vec<LibraryRecord> = Vec::with_capacity(file_count as usize);

            for _ in 0..file_count {
                let offset = reader.read_u32()?;
                let size = reader.read_u32()?;
                let zip_size = reader.read_u32()?;
                let mut zipped = reader.read_bool()?;
                let crc32 = reader.read_u32()?;
                let file_name = reader.read_string()?.trim_end_matches('\x00').to_owned();

                // Some of them are falsely marked as zipped but aren't actually
                if file_name.ends_with(".wav") || file_name.ends_with(".ogg") || file_name.ends_with(".mp3") {
                    zipped = false;
                }

                // Add the FileRecord to the HashMap with properties
                files.push(LibraryRecord {
                    offset,
                    size,
                    zip_size,
                    zipped,
                    crc32,
                    file_name,
                });
            }

            println!("\tFound {file_count} records");

            Ok(Self {
                version,
                file_count,
                files,
                mmap,
            })
        } else {
            panic!("No valid KIWAD header recognized!");
        }
    }

    pub fn extract<P: AsRef<Path> + Sync>(&mut self, out_dir: &mut P, filter: Option<&HashSet<String>>) -> io::Result<()> {
        create_dir_all(&out_dir)?;

        let records: Vec<&LibraryRecord> = if let Some(filter) = filter {
            self.files.iter().filter(|r| filter.contains(&r.file_name)).collect()
        } else {
            self.files.iter().collect()
        };

        // Pre-Create all directories
        let dirs: HashSet<_> = records
            .iter()
            .filter_map(|rec| Path::new(&rec.file_name).parent())
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| out_dir.as_ref().join(p))
            .collect();

        for dir in &dirs {
            create_dir_all(dir)?;
        }

        let buf: &[u8] = &self.mmap;
        records.par_iter().try_for_each(|rec| -> io::Result<()> {
            let dst = out_dir.as_ref().join(&rec.file_name);

            let off = rec.offset as usize;
            let len = if rec.zipped { rec.zip_size } else { rec.size } as usize;
            let chunk = &buf[off..off + len];

            if Self::is_empty(chunk) {
                File::create(&dst)?;
                return Ok(());
            }

            let mut out = File::create(&dst)?;
            if rec.zipped {
                let mut decoder = ZlibDecoder::new(chunk);
                io::copy(&mut decoder, &mut out)?;
            } else {
                out.write_all(chunk)?;
            }

            Ok(())
        })?;

        Ok(())
    }

    /// Check if a `Vec<u8>` is the magic header `KIWAD`
    #[inline]
    fn is_magic_header(input_bytes: &[u8]) -> bool {
        input_bytes == MAGIC_HEADER
    }

    /// Returns `true` if the Vec only contains NULL bytes (which are impossible to inflate) or len is 0
    #[inline]
    #[must_use]
    pub fn is_empty(slice: &[u8]) -> bool {
        slice.is_empty() || slice.iter().all(|&byte| byte == 0)
    }
}
