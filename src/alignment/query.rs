use bam::{Record, Region};

pub fn query(file: &str, chr: u32, start: u32, end: u32) -> Option<Vec<Record>> {
    let mut reader = bam::IndexedReader::from_path(file).unwrap();
    let viewer = reader.fetch(&Region::new(chr, start, end)).unwrap();
    let mut records = Vec::<Record>::new();
    for record in viewer {
        records.push(record.unwrap());
    }
    Some(records)
}
