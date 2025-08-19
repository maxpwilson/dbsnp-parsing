use anyhow::Result;

mod getdbsnp;
mod download;
mod checkmd5;

use getdbsnp::{dbsnp_latest_release, download_dbsnp};

fn main() -> Result<()> {
    download_dbsnp(true)?;
    Ok(())
}
