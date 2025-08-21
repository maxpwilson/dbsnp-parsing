#![allow(dead_code)]

use crate::checkmd5::md5_hash_file_verbose;
use crate::download::Download;
use anyhow::Result;

const LOCALPATH: &str = "../downloads/";
const DBSNPSERVER: &str = "https://ftp.ncbi.nih.gov/snp/latest_release/";
const RELEASEFILE: &str = "release_notes.txt";
const VERSIONLOC: usize = 2;
const VCFFILE: &str = "GCF_000001405.40.gz";

fn vcf_server() -> Option<String> {
    Some(DBSNPSERVER.to_string() + "VCF/")
}

async fn dbsnp_release_info() -> Result<String> {
    let version_dl = Download::new(
        RELEASEFILE.to_string(),
        DBSNPSERVER.to_string(),
        LOCALPATH.to_string(),
    );
    let release_notes = version_dl.get_text().await?;
    Ok(release_notes)
}

pub async fn dbsnp_latest_release() -> Result<String> {
    let release_info = dbsnp_release_info().await?;
    let version = release_info
        .split_whitespace()
        .nth(VERSIONLOC)
        .unwrap_or("NA");
    Ok(version.to_string())
}

fn md5_dls(filename: String, server: String, localpath: String) -> Option<(Download, Download)> {
    Some((
        Download::new(filename.clone(), server.clone(), localpath.clone()),
        Download::new(filename + ".md5", server, localpath),
    ))
}
#[tokio::main]
async fn download_check_file(dl: Download, dl_md5: Download, force: bool) -> Result<()> {
    let md5_info = dl_md5.get_text().await?;
    assert_eq!(dl.filename, md5_info.split_whitespace().nth(1).unwrap());
    let md5_dl_hash = md5_info.split_whitespace().nth(0).unwrap();
    if !dl.is_local().unwrap_or(false) || force {
        println!("Downloading {}", dl.filename);
        dl.verbose_download(None).await?;
    }
    println!("Downloaded {}", dl.localfile().unwrap());
    println!("Calculating checksum");
    let md5_calc_hash = md5_hash_file_verbose(dl.localfile().unwrap(), None).unwrap();
    assert_eq!(md5_dl_hash, md5_calc_hash);
    println!("Successfully validated file");
    Ok(())
}
pub async fn download_dbsnp(force: bool) -> Result<()> {
    let (vcf_dl, vcf_md5_dl) = md5_dls(
        VCFFILE.to_string(),
        vcf_server().unwrap(),
        LOCALPATH.to_string(),
    )
    .unwrap();
    let (tbi_dl, tbi_md5_dl) = md5_dls(
        VCFFILE.to_string() + ".tbi",
        vcf_server().unwrap(),
        LOCALPATH.to_string(),
    )
    .unwrap();
    let dbsnp_version = dbsnp_latest_release().await?;
    println!("Latest dbSNP version: {}", dbsnp_version);
    println!("Force re-download set to: {}", force);
    download_check_file(vcf_dl, vcf_md5_dl, force)?;
    download_check_file(tbi_dl, tbi_md5_dl, force)?;
    Ok(())
}
