#![allow(dead_code)]

use crate::alignment::{KNOWNFILE, LOCALPATH, MODELFILE};
use crate::checkmd5::md5_hash_file_verbose;
use crate::download::Download;
use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar};
use lazy_static::lazy_static;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::RwLock;

const ANNOTSERVER: &str = "https://ftp.ncbi.nlm.nih.gov/genomes/refseq/vertebrate_mammalian/Homo_sapiens/reference/GCF_000001405.40_GRCh38.p14/";
const MD5FILE: &str = "md5checksums.txt";

fn annot_folder() -> Option<String> {
    Some(ANNOTSERVER.to_string() + "RefSeq_transcripts_alignments/")
}

fn annot_dls() -> Option<Vec<Download>> {
    Some(vec![
        Download::new(
            MD5FILE.to_string(),
            ANNOTSERVER.to_string(),
            LOCALPATH.to_string(),
        ),
        Download::new(
            KNOWNFILE.to_string(),
            annot_folder().unwrap(),
            LOCALPATH.to_string(),
        ),
        Download::new(
            KNOWNFILE.to_string() + ".bai",
            annot_folder().unwrap(),
            LOCALPATH.to_string(),
        ),
        Download::new(
            MODELFILE.to_string(),
            annot_folder().unwrap(),
            LOCALPATH.to_string(),
        ),
        Download::new(
            MODELFILE.to_string() + ".bai",
            annot_folder().unwrap(),
            LOCALPATH.to_string(),
        ),
    ])
}

async fn get_hash(filename: &str) -> Option<String> {
    let ref_file = File::open(LOCALPATH.to_string() + MD5FILE).unwrap();
    let reader = BufReader::new(ref_file);
    for line in reader.lines() {
        let line = line.unwrap();
        let hash = line.split_whitespace().nth(0).unwrap();
        let file = line.split_whitespace().nth(1).unwrap();
        if file == filename {
            return Some(hash.to_string());
        }
    }
    None
}

#[tokio::main]
pub async fn download_alignments() -> Result<()> {
    let dls = annot_dls().context("Failed to get alignment downloads")?;
    let mut handles = vec![];
    lazy_static! {
        pub static ref MP: RwLock<MultiProgress> = RwLock::new(MultiProgress::new());
    }
    for dl in dls {
        handles.push(tokio::task::spawn(async move {
            let pb = MP.write().unwrap().add(ProgressBar::new(0));
            dl.verbose_download(Some(pb)).await.unwrap();
            if dl.filename != MD5FILE.to_string() {
                let hash = match get_hash(
                    &("./RefSeq_transcripts_alignments/".to_string() + &dl.filename),
                )
                .await
                {
                    Some(hash) => hash,
                    None => {
                        eprintln!("Failed to get hash for {}", dl.filename);
                        return;
                    }
                };
                let calc_hash = {
                    let pb = MP.write().unwrap().add(ProgressBar::new(0));
                    md5_hash_file_verbose(LOCALPATH.to_string() + &dl.filename, Some(pb)).unwrap()
                };
                if calc_hash != hash {
                    println!("hash {} calced {}", hash, calc_hash);
                    eprintln!("Hash mismatch for {}", dl.filename);
                }
            };
        }));
    }
    futures::future::join_all(handles).await;
    MP.write().unwrap().clear().unwrap();
    println!("Finished downloading");
    Ok(())
}
