use anyhow::Result;
use lazy_static::lazy_static;

mod get;
mod query;

const KNOWNFILE: &str = "GCF_000001405.40_GRCh38.p14_knownrefseq_alns.bam";
const MODELFILE: &str = "GCF_000001405.40_GRCh38.p14_modelrefseq_alns.bam";
const LOCALPATH: &str = "../downloads/";

lazy_static! {
    pub static ref ALIGNMENTS: Alignments = Alignments::new(
        LOCALPATH.to_string() + MODELFILE,
        LOCALPATH.to_string() + KNOWNFILE
    );
}

pub struct Alignments {
    pub modelfile: String,
    pub knownfile: String,
}
impl Alignments {
    fn new(modelfile: String, knownfile: String) -> Alignments {
        Alignments {
            modelfile: modelfile,
            knownfile: knownfile,
        }
    }
    pub fn check_files(&self) {}
    pub fn download_files(&self) -> Result<()> {
        get::download_alignments()?;
        Ok(())
    }
    pub fn query(&self, chr: u32, start: u32, end: u32) {
        let records = query::query(&self.modelfile, chr, start, end).unwrap();
        for record in records {
            println!("{:?}", record);
        }
    }
}
