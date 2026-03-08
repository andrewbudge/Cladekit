use clap::Args;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Args)]
pub struct GetheadersArgs {
    /// input fasta file, read from stdin if no file is provided
    pub input: Option<String>,

    /// print only unique headers
    #[arg(short, long)]
    pub unique: bool,
}

pub fn run(args: GetheadersArgs) {
    // open the provided file
    let filename = args.input.expect("no input file provided");
    let file = File::open(&filename).expect("Unable to open file");

    // create the reader
    let reader = BufReader::new(file);

    // track seen headers for unique mode
    let mut seen = HashSet::new();

    // read the file and print only the lines that start with ">" (the seq ids)
    for line in reader.lines() {
        let line = line.expect("Could not read line");
        if line.starts_with('>') {
            let header = &line[1..];
            if args.unique {
                if seen.insert(header.to_string()) {
                    println!("{}", header);
                }
            } else {
                println!("{}", header);
            }
        }
    }
}
