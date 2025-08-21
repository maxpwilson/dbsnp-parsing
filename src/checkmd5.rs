#![allow(dead_code)]

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use md5::{Digest, Md5};
use std::fs;
use std::io::{BufReader, Read, copy};

pub fn md5_hash_file(filepath: String) -> Result<String> {
    let mut file = fs::File::open(&filepath)?;
    let mut hasher = Md5::new();
    let _n = copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}
pub fn md5_hash_file_verbose(filepath: String, multipb: Option<MultiProgress>) -> Result<String> {
    let file = fs::File::open(&filepath)?;
    let filesize = file.metadata().unwrap().len();
    let mut hasher = Md5::new();
    let mut source = BufReader::new(file);
    let mut buffer = [0u8; 8192];
    let pb = match multipb {
        Some(p) => p.add(ProgressBar::new(filesize)),
        None => ProgressBar::new(filesize),
    };
    pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
            )
            .unwrap()
            .progress_chars("##-"),
        );
    loop {
        let bytes_read = source.read(&mut buffer)?;
        hasher.update(&buffer[..bytes_read]);
        if bytes_read == 0 {
            break;
        }
        pb.inc(bytes_read as u64);
    }
    pb.finish_with_message("Hashing complete");
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}
