use binary_rw::{BinaryError, BinaryReader, Endian};
use flate2::DecompressError;
use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::HashMap,
    fs::{self, create_dir_all},
    io::{Cursor, Read},
    ops::Add,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub offset: u32,
    pub size: u32,
    pub zip_size: u32,
    pub zipped: bool,
    pub crc32: u32,
    pub file_name: String,
}

#[derive(Debug)]
pub struct WadRework<'a> {
    pub version: u32,
    pub file_count: u32,
    pub files: HashMap<String, FileRecord>,
    pub buffer: &'a mut [u8],
}

impl<'a> WadRework<'a> {
    pub fn new(buffer: &'a mut Vec<u8>) -> Result<Self, BinaryError> {
        let mut reader = BinaryReader::new_vec(buffer, Endian::Little);

        let header = &reader.read_bytes(5)?;

        if !Self::is_magic_header(header) {
            panic!("No valid KiWAD file was recognized!");
        } else {
            let version = reader.read_u32()?;
            let file_count = reader.read_u32()?;

            println!("\tFileCount: {file_count}");
            println!("\tVersion: {version}");

            if version >= 2 {
                reader.read_bytes(1)?;
            }

            let mut files: HashMap<String, FileRecord> = HashMap::new();

            for _ in 0..file_count {
                let offset = reader.read_u32()?;
                let size = reader.read_u32()?;
                let zip_size = reader.read_u32()?;
                let mut zip = reader.read_bool()?;
                let crc = reader.read_u32()?;
                let file_name = reader.read_big_string()?.to_string().replace("\x00", "");

                // Some of them are falsely marked as zipped but aren't actually
                if file_name.ends_with(".wav")
                    || file_name.ends_with(".ogg")
                    || file_name.ends_with(".mp3")
                {
                    zip = false
                }

                // Add the FileRecord to the HashMap with properties
                files.insert(
                    file_name.clone(),
                    FileRecord {
                        zip_size,
                        crc32: crc,
                        file_name,
                        zipped: zip,
                        offset,
                        size,
                    },
                );
            }

            Ok(Self {
                file_count,
                files,
                version,
                buffer,
            })
        }
    }

    pub fn open_all_files(&mut self, mut path: &mut PathBuf) -> () {
        self.files
            .par_iter()
            .into_par_iter()
            .for_each(|(file_name, file_record)| {
                let mut buffer: Vec<u8> = Vec::with_capacity(file_record.size as usize);
                let mut cursor: Cursor<&&mut [u8]> = Cursor::new(&self.buffer);

                let data = {
                    let mut result = vec![
                        0;
                        match file_record.zipped {
                            true => file_record.zip_size,
                            false => file_record.size,
                        } as usize
                    ];

                    cursor.set_position(file_record.offset as u64);
                    cursor.read_exact(&mut result).unwrap();
                    result
                };

                if file_record.zipped {
                    if !Self::is_empty(&data) {
                        let mut decompressor = flate2::Decompress::new(true);
                        decompressor
                            .decompress_vec(&data[..], &mut buffer, flate2::FlushDecompress::Finish)
                            .unwrap();
                    } else {
                        buffer.clear();
                    }
                } else {
                    buffer = data.to_vec();
                }

                let path = &mut path.join(file_name);
                create_dir_all(&path.parent().unwrap()).unwrap();
                fs::write(&path, &buffer)
                    .unwrap_or_else(|e| eprintln!("Could not write to file! {}", e)); // Write to the file

                buffer.clear();
            });
    }

    pub fn read_file(&mut self, name: &str) -> Result<Vec<u8>, DecompressError> {
        let file_record = self
            .files
            .get(name)
            .expect("File does not exist inside of wad!");

        let mut buffer: Vec<u8> = Vec::with_capacity(file_record.size as usize);
        let mut cursor: Cursor<&&mut [u8]> = Cursor::new(&self.buffer);

        let data = {
            let mut result = vec![
                0;
                match file_record.zipped {
                    true => file_record.zip_size,
                    false => file_record.size,
                } as usize
            ];

            cursor.set_position(file_record.offset as u64);
            cursor.read_exact(&mut result).unwrap();
            result
        };

        if file_record.zipped {
            if !Self::is_empty(&data) {
                let mut decompressor = flate2::Decompress::new(true);
                decompressor.decompress_vec(
                    &data[..],
                    &mut buffer,
                    flate2::FlushDecompress::Finish,
                )?;
            } else {
                buffer.clear();
            }
        } else {
            buffer = data.to_vec();
        }

        Ok(buffer)
    }

    /// Check if a `Vec<u8>` is the magic header `KIWAD`
    fn is_magic_header(input_bytes: &[u8]) -> bool {
        input_bytes == b"KIWAD"
    }

    /// Returns `true` if the Vec only contains NULL bytes (which are impossible to inflate) or len is 0
    pub fn is_empty(slice: &[u8]) -> bool {
        slice.is_empty() || slice.iter().all(|&byte| byte == 0)
    }
}
