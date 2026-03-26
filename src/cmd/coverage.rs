use clap::Args;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Args)]
pub struct CoverageArgs {
    /// Provenance TSV file from concat
    pub tsv: String,
}

pub fn run(args: CoverageArgs) {
    // TODO: read the TSV, count present vs MISSING per taxon, print summary
    let file = File::open(&args.tsv).expect("Failed to open TSV");
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let header = lines
        .next()
        .expect("empty file")
        .expect("Failed to read header");
    for line in lines {
        let line = line.expect("Failed to read line");
        let fields: Vec<&str> = line.split('\t').collect();
        let total_loci = header.split('\t').count() - 1;
        let present_count = fields[1..].iter().filter(|f| **f != "MISSING").count();
        let missing_pct = (present_count as f64 / total_loci as f64) * 100.0;
        println!("{} \t {}%", fields[0], missing_pct);
    }
}
