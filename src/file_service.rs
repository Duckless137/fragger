use crate::PrintErr;
use std::{
    fs, io::{Read, Write}, os::windows::fs::MetadataExt, path::PathBuf
};

use super::ExpectedErrors;

// 128 MB
pub const MAX_BUFFER_SIZE: u64 = 128 * 1024 * 1024;

// 4 GB
pub const MAX_CHUNK_SIZE: u64 = 4 * 1024 * 1024 * 1024 * 1024;

// 1 KB
pub const MIN_CHUNK_SIZE: u64 = 1024;

// 4 TB
pub const MAX_FILE_SIZE: u64 = 4 * 1024 * 1024 * 1024 * 1024;

pub fn split_file(path: PathBuf, chunk_size: u64) {
    // Makes sure chunk sizes are within limit (They shouldn't be due to app.rs's restrictions)
    if chunk_size > MAX_CHUNK_SIZE {
        ExpectedErrors::FileTooBig(chunk_size, MAX_CHUNK_SIZE).print();
        return;
    }

    if chunk_size < MIN_CHUNK_SIZE {
        ExpectedErrors::FileTooSmall(chunk_size, MAX_CHUNK_SIZE).print();
        return;
    }

    // Makes sure file is
    //  a. Within file size limits
    //  b. Larger than chunk size
    if let Ok(file_metadata) = std::fs::metadata(&path) {
        if file_metadata.file_size() < chunk_size {
            ExpectedErrors::FileLargerThanChunkSize.print();
            return;
        } else if file_metadata.file_size() > MAX_FILE_SIZE {
            ExpectedErrors::GiantFile.print();
            return;
        }
    } else {
        ExpectedErrors::CouldNotReadFile(path.display().to_string()).print();
        return;
    };

    // Gets the path's parent
    let dir = path.parent();
    if let Some(dir) = dir {
        let file_stem = path.file_stem();
        if file_stem.is_none() {
            ExpectedErrors::NoName.print();
            return;
        }

        let file_stem = file_stem.print_and_unwrap().to_str();
        if file_stem.is_none() {
            ExpectedErrors::NoName.print();
            return;
        }

        // Creates a new dir to store fragment files
        let new_dir_name = format!("{}/{}", dir.display(), file_stem.print_and_unwrap());

        println!("{:?}", path.file_stem().print_and_unwrap());
        let new_dir = std::fs::create_dir_all(&new_dir_name);

        if new_dir.is_err() {
            ExpectedErrors::CouldNotMakeDir(new_dir_name).print();
            return;
        }

        if let Ok(file) = std::fs::File::open(&path) {
            {
                // The byte array ID for the name file is literally just [0, 0, 0, 0], which is fairly convenient
                let metadata_buf: [u8; 4] = [0; 4];
                let file_name: String = format!("{}/filedata.frag", new_dir_name);
                if std::fs::File::create(&file_name).is_err() {
                    ExpectedErrors::CouldNotCreateFile(file_name).print();
                    return;
                };

                if let Ok(mut file) = fs::OpenOptions::new().append(true).open(&file_name) {
                    if file.write(&metadata_buf).is_err() {
                        ExpectedErrors::CouldNotWriteFile(file_name).print();
                        return;
                    };

                    if let Some(base_name) = path.file_name() {
                        if file.write(base_name.as_encoded_bytes()).is_err() {
                            ExpectedErrors::CouldNotWriteFile(file_name).print();
                            return;
                        };
                    } else {
                        ExpectedErrors::NoName.print();
                        return;
                    }
                } else {
                    ExpectedErrors::CouldNotReadFile(file_name).print();
                    return;
                }
            }
            // i: File ID
            let mut i: u32 = 1;
            let mut reader = std::io::BufReader::new(file);
            let mut buffer:Box<[u8]>  = vec![0; chunk_size as usize - 4].into_boxed_slice();
            loop {
                // Reads file into buffer
                if let Ok(bytes_read) = reader.read(&mut buffer) {
                    if bytes_read == 0 {
                        break;
                    } else {
                        // Gets only the used parts of the buffer
                        let buffslice = &buffer[0..bytes_read];

                        // Makes new fragment file's name
                        let file_name = format!("{}/split_file_{}.frag", new_dir_name, i);
                        if std::fs::File::create(&file_name).is_err() {
                            ExpectedErrors::CouldNotCreateFile(file_name).print();
                            return;
                        }

                        // Creates new fragment file
                        if let Ok(mut fragment) =
                            std::fs::OpenOptions::new().append(true).open(&file_name)
                        {
                            // File ID bytes
                            let file_num_in_bytes = into_u8(i);
                            // Write file ID bytes
                            if fragment.write_all(&file_num_in_bytes).is_err() {
                                ExpectedErrors::CouldNotWriteFile(file_name).print();
                                return;
                            };

                            // Writes used bytes into file
                            if fragment.write_all(buffslice).is_err() {
                                ExpectedErrors::CouldNotWriteFile(file_name).print();
                                return;
                            };
                        } else {
                            ExpectedErrors::UnreadableFile.print();
                        };
                        i += 1;
                    }
                } else {
                    ExpectedErrors::CouldNotReadFile(path.display().to_string()).print();
                    return;
                };
            }
        } else {
            ExpectedErrors::UnreadableFile.print();
        }
    } else {
        ExpectedErrors::NoParent.print();
    }
}

pub fn combine_files(path: PathBuf) {
    let mut file_paths: Vec<(PathBuf, u32)> = Vec::new();

    if let Ok(dir) = path.read_dir() {
        for entry in dir {
            if let Ok(entry) = entry {
                // Gets .frag files
                if entry.path().extension() == Some(std::ffi::OsStr::new("frag"))
                    && !entry.path().is_dir()
                {
                    // Buffer for ID bytes
                    let mut order_buf: [u8; 4] = [0, 0, 0, 0];
                    if let Ok(mut file) = std::fs::File::open(&entry.path()) {
                        // Writes ID bytes to buffer
                        if file.read_exact(&mut order_buf).is_err() {
                            ExpectedErrors::CouldNotReadFile(entry.path().display().to_string())
                                .print();
                        }
                    } else {
                        ExpectedErrors::CouldNotReadFile(entry.path().display().to_string())
                            .print();
                        return;
                    }
                    let append = (
                        entry.path(),
                        /*Turns [u8] ID byte array in u32 ID*/ into_u32(order_buf),
                    );

                    file_paths.push(append);
                } else {
                    continue;
                }
            } else {
                ExpectedErrors::BadFiles.print();
            }
        }
    } else {
        ExpectedErrors::CouldNotReadDir(path.display().to_string()).print();
        return;
    }

    // Sorts files by ID bytes
    file_paths.sort_by(|a, b| a.1.cmp(&b.1));

    let mut file_name = if path.parent().is_none() {
        ExpectedErrors::NullPath.print();
        return;
    } else {
        PathBuf::from(path.parent().print_and_unwrap())
    };

    let data_file_path = file_paths.remove(0).0;
    let data_file = fs::File::open(&data_file_path);
    if data_file.is_err() {
        ExpectedErrors::CouldNotReadFile(data_file_path.display().to_string()).print();
        return;
    }
    let mut data_file = data_file.print_err().print_and_unwrap();
    {
        let mut buffer = Vec::new();
        if data_file.read_to_end(&mut buffer).is_err() {
            ExpectedErrors::CouldNotReadFile(data_file_path.display().to_string()).print();
            return;
        }
        let read_name: &[u8] = &buffer[4..buffer.len()];
        if let Ok(name_as_str) = std::str::from_utf8(read_name) {
            file_name = file_name.join(name_as_str);
        } else {
            ExpectedErrors::InvalidString.print();
            return;
        }
    }

    // Create target file
    if fs::File::create(&file_name).is_err() {
        ExpectedErrors::CouldNotCreateFile(file_name.display().to_string()).print();
        return;
    }

    // Test for permissions
    if fs::write(&file_name, b"").is_err() {
        ExpectedErrors::CouldNotWriteFile(file_name.display().to_string()).print();
        return;
    };

    // Open target file
    if let Ok(mut main_file) = fs::OpenOptions::new().append(true).open(&file_name) {
        
        // Allocates the buffer based on the size of the first file.
        // This is somewhat risky because it assumes no files have been
        // tampered with, but it stops the program from reallocating 
        // the buffer for each file- which would be terrible for perfomance
        // because of how slow the heap is.
        let buffer_size: usize;
        if let Ok(file_metadata) = std::fs::metadata(&file_paths[0].0) {
            buffer_size = file_metadata.file_size().clamp(0, MAX_BUFFER_SIZE) as usize;
        } else {
            ExpectedErrors::CouldNotReadFile(path.display().to_string()).print();
            return;
        };

        // Idealy, I would like to get rid of this buffer, since it uses recources
        let mut buffer = vec![0;buffer_size].into_boxed_slice();
        
        // Read fragment files (skip name file)
        for (file_path, _order) in file_paths.into_iter() {
            if let Ok(mut fragment) = fs::File::open(&file_path) {
                loop {
                    let bytes_read = fragment.read(&mut buffer);
                
                    // Reads the fragment file into the buffer
                    if let Ok(bytes_read) = bytes_read {
                        if bytes_read < 1 {
                            break;
                        }
    
                        // Disregard ID bytes
                        let content = &buffer[4..bytes_read];
    
                        // Writes buffer to target file
                        if main_file.write_all(content).is_err() {
                            ExpectedErrors::CouldNotWriteFile(file_path.display().to_string()).print();
                            return;
                        }   
                    } else {                        
                        ExpectedErrors::CouldNotReadFile(file_path.display().to_string()).print();
                        return;
                    }
                }
            } else {
                ExpectedErrors::UnreadableFile.print();
            }
        }
    } else {
        ExpectedErrors::CouldNotReadFile(file_name.display().to_string()).print();
    }
}

pub fn into_u8(num: u32) -> [u8; 4] {
    let mut res: [u8; 4] = [0, 0, 0, 0];
    let iter = num.checked_ilog(256);
    if iter.is_none() {
        res[0] = num as u8;
    } else {
        for i in 0..(iter.print_and_unwrap() + 1) {
            res[i as usize] = (num >> (i * 8)) as u8;
        }
    }
    res
}

pub fn into_u32(num: [u8; 4]) -> u32 {
    let mut res: u32 = 0;
    for (i, val) in num.into_iter().enumerate() {
        res += (val as u32) << (i * 8);
    }
    res
}
