use anyhow::Result;
fn main() -> Result<()> {
    snp_parsing::alignment::ALIGNMENTS.query(1, 10000, 100000);
    Ok(())
}
