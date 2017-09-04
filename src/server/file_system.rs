use std::path::{Path, PathBuf};
use std::io::Write;
use std::io;
use std::fs::File;
use std::fs::metadata;
use chrono::{DateTime, TimeZone};
use chrono::offset::Utc;
use std::cmp::Ordering;
use std::time::SystemTime;
use super::content_manager::{ContentHandle, ContentManager};

pub struct FileSystemAdapter {
    root: PathBuf
}

impl FileSystemAdapter {
    pub fn new(root: &Path) -> FileSystemAdapter {
        return FileSystemAdapter {
            root: root.to_path_buf()
        }
    }
}

impl ContentManager<FileHandle> for FileSystemAdapter {
    fn find_content(&self, url: &String, accepts_gzip: bool) -> Option<FileHandle> {
        println!("loading {} gzip={}", url, accepts_gzip);
        let path = if url.starts_with('/') {
            let mut temp = url.clone();
            temp.remove(0);
            temp
        } else {
            url.clone()
        };

        match get_file_stats(&self.root.join(path.clone())) {
            Some(FileStats(mod_date, raw_len)) => {
                if accepts_gzip {
                    let mut gz_path = path.clone();
                    gz_path.push_str(".gz");
                    let gz_file_path = self.root.join(gz_path);
                    if let Some(FileStats(_, gzipped_len)) = get_file_stats(&gz_file_path) {
                        match File::open(gz_file_path) {
                            Ok(file) => {
                                return Some(FileHandle::new(mod_date.unwrap(), gzipped_len, true, file))
                            },
                            Err(_) => {}
                        }
                    }
                }

                let file_path = self.root.join(path);
                println!("file_path={}", file_path.display());
                match File::open(file_path) {
                    Ok(file) => Some(FileHandle::new(mod_date.unwrap(), raw_len, false, file)),
                    Err(e) => {
                        eprintln!("Error opening {}:{}", url, e);
                        None
                    }
                }
            },
            None => None
        }
    }
}

struct FileStats(Option<SystemTime>, u64);

fn get_file_stats(path: &PathBuf) -> Option<FileStats> {
    match metadata(path) {
        Ok(md) => Some(FileStats(md.modified().ok(), md.len())),
        Err(e) => {
            eprintln!("Error finding file {}:{}", path.display(), e);
            return None
        }
    }
}
pub struct FileHandle {
    mod_date: DateTime<Utc>,
    content_length: u64,
    gzipped: bool,
    file: File
}

impl FileHandle {
    fn new(mod_date: SystemTime, content_len: u64, gzipped: bool, file: File) -> FileHandle {
        FileHandle {
            mod_date: DateTime::from(mod_date),
            content_length: content_len,
            gzipped: gzipped,
            file: file,
        }
    }
}

impl ContentHandle for FileHandle {
    fn write_to(&mut self, writer: &mut Write) {
        io::copy(&mut self.file, writer).expect("Error while copying file\n");
    }

    fn is_gzipped(&self) -> bool {
        self.gzipped
    }

    fn is_mod_since<TZ: TimeZone>(&self, other: &DateTime<TZ>) -> bool {
        let mod_time = self.mod_time();

        let or = other.timestamp().cmp(&mod_time.timestamp());

        match or {
            Ordering::Greater | Ordering::Equal => true,
            Ordering::Less => false
        }
    }

    fn mod_time(&self) -> &DateTime<Utc> {
        &self.mod_date
    }
    fn content_length(&self) -> u64 {
       self.content_length
    }
}