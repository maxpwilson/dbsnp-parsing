use anyhow::Result;

mod checkmd5;
mod download;
mod getannotations;
mod getdbsnp;

fn main() -> Result<()> {
    getannotations::download_annotations()?;
    println!("Done from main");
    Ok(())
}
