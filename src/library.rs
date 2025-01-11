use binary_modifier::{BinaryError, BinaryReader, Endian};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs::{create_dir_all, File},
    io::{BufWriter, Cursor, Read, Write},
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub struct LibraryRecord {
    pub offset: u32,
    pub size: u32,
    pub zip_size: u32,
    pub zipped: bool,
    pub crc32: u32,
    pub file_name: String,
}

#[derive(Debug)]
pub struct Library<'a> {
    pub version: u32,
    pub file_count: u32,
    pub files: Vec<LibraryRecord>,
    pub buffer: &'a mut [u8],
}

const MAGIC_HEADER: &[u8; 5] = b"KIWAD";

impl<'a> Library<'a> {
    /// # Panics
    ///
    /// Will panic if the file doesn't contain a `KIWAD` header!
    pub fn new(buffer: &'a mut Vec<u8>) -> Result<Self, BinaryError> {
        let mut reader = BinaryReader::new_vec(buffer, Endian::Little);
        let header = &reader.read_bytes(5)?;

        if Self::is_magic_header(header) {
            let version = reader.read_u32()?;
            let file_count = reader.read_u32()?;

            if version >= 2 {
                reader.read_bytes(1)?;
            }

            let mut files: Vec<LibraryRecord> = Vec::with_capacity(file_count as usize);

            for _ in 0..file_count {
                let offset = reader.read_u32()?;
                let size = reader.read_u32()?;
                let zip_size = reader.read_u32()?;
                let mut zipped = reader.read_bool()?;
                let crc32 = reader.read_u32()?;
                let file_name = reader.read_big_string()?.to_string().replace('\x00', "");

                // Some of them are falsely marked as zipped but aren't actually
                if file_name.ends_with(".wav") || file_name.ends_with(".ogg") || file_name.ends_with(".mp3") {
                    zipped = false;
                }

                // Add the FileRecord to the HashMap with properties
                files.push(LibraryRecord {
                    zip_size,
                    crc32,
                    file_name,
                    zipped,
                    offset,
                    size,
                });
            }

            println!("\tFound {file_count} files in WAD, version {version}!");

            Ok(Self {
                version,
                file_count,
                files,
                buffer,
            })
        } else {
            panic!("No valid KIWAD header recognized!");
        }
    }

    pub fn open_all_files(&mut self, path: &mut PathBuf) {
        self.files.par_iter().for_each(|file_record| {
            let mut buffer = Vec::with_capacity(file_record.size as usize);
            let mut cursor = Cursor::new(&self.buffer);

            let data = {
                let mut result = vec![
                    0;
                    if file_record.zipped {
                        file_record.zip_size
                    } else {
                        file_record.size
                    } as usize
                ];

                cursor.set_position(u64::from(file_record.offset));
                cursor.read_exact(&mut result).unwrap();
                result
            };

            if file_record.zipped {
                if Self::is_empty(&data) {
                    buffer.clear();
                } else {
                    let mut decompressor = flate2::Decompress::new(true);
                    decompressor
                        .decompress_vec(&data[..], &mut buffer, flate2::FlushDecompress::Finish)
                        .unwrap();
                }
            } else {
                buffer = data
            }

            // Prepare the file path
            let path = path.join(&file_record.file_name);
            create_dir_all(path.parent().unwrap()).unwrap();

            // Write to the file
            let mut file = BufWriter::new(File::create(&path).unwrap());
            file.write_all(&buffer).unwrap();
        });
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
