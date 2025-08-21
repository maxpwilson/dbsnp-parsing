#![allow(dead_code)]

use crate::checkmd5::md5_hash_file_verbose;
use crate::download::Download;
use anyhow::{Context, Result};
use indicatif::MultiProgress;
use std::fs::File;
use std::io::{BufRead, BufReader};

const LOCALPATH: &str = "../downloads/";
const ANNOTSERVER: &str = "https://ftp.ncbi.nlm.nih.gov/genomes/refseq/vertebrate_mammalian/Homo_sapiens/reference/GCF_000001405.40_GRCh38.p14/";
const MD5FILE: &str = "md5checksums.txt";
const KNOWNFILE: &str = "GCF_000001405.40_GRCh38.p14_knownrefseq_alns.bam";
const MODELFILE: &str = "GCF_000001405.40_GRCh38.p14_modelrefseq_alns.bam";

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
pub async fn download_annotations() -> Result<()> {
    let dls = annot_dls().context("Failed to get annotation downloads")?;
    let mut handles = vec![];
    let m = MultiProgress::new();
    for dl in dls {
        let m_clone = m.clone();
        handles.push(tokio::task::spawn(async move {
            dl.verbose_download(Some(m_clone.clone())).await.unwrap();
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
                let calc_hash =
                    md5_hash_file_verbose(LOCALPATH.to_string() + &dl.filename, Some(m_clone))
                        .unwrap();
                if calc_hash != hash {
                    println!("hash {} calced {}", hash, calc_hash);
                    eprintln!("Hash mismatch for {}", dl.filename);
                }
            };
        }));
    }
    m.clear().unwrap();
    futures::future::join_all(handles).await;
    println!("Finished downloading");
    Ok(())
}
