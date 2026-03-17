use clap::Args;
use phylo::parse_fasta;
use std::path::Path;

#[derive(Args)]
pub struct StatsArgs {
    /// FASTA alignment files
    pub files: Vec<String>,
}

/// Count variable and parsimony-informative sites in an alignment.
/// Variable: column has at least 2 different bases (ignoring gaps/N).
/// Parsimony-informative: column has at least 2 different bases, each appearing at least twice.
fn count_informative_sites(
    sequences: &std::collections::HashMap<String, String>,
    length: usize,
) -> (usize, usize) {
    let mut variable = 0;
    let mut informative = 0;
    let seqs: Vec<&String> = sequences.values().collect();

    for i in 0..length {
        // count bases at this column: [A, T, C, G]
        let mut counts = [0usize; 4];
        for seq in &seqs {
            let base = seq.as_bytes()[i] as char;
            match base {
                'A' => counts[0] += 1,
                'T' => counts[1] += 1,
                'C' => counts[2] += 1,
                'G' => counts[3] += 1,
                _ => {} // ignore gaps and N
            }
        }

        let mut bases_present = 0;
        let mut bases_twice = 0;
        for count in &counts {
            if *count > 0 {
                bases_present += 1;
            }
            if *count >= 2 {
                bases_twice += 1;
            }
        }

        if bases_present > 1 {
            variable += 1;
        }
        if bases_twice >= 2 {
            informative += 1
        }
    }

    (variable, informative)
}

pub fn run(args: StatsArgs) {
    // print TSV header row
    // columns: file, sequences, length, gc_pct, missing_pct, variable, variable_pct, informative, informative_pct
    println!("file\tsequences\tlength\tgc_pct\tmissing_pct\tvariable\tvariable_pct\tinformative\tinformative_pct");

    for file in args.files {
        let (sequences, length) = parse_fasta(&file, false).expect("Failed to parse fasta file");
        let num_sequences = sequences.len();
        let mut gc_count = 0;
        let mut missing_count = 0;
        let mut total_nucleotides = 0;
        for sequence in sequences.values() {
            for nucleotide in sequence.chars() {
                total_nucleotides += 1;
                match nucleotide {
                    'G' | 'C' => gc_count += 1,
                    '-' | 'N' => missing_count += 1,
                    _ => {}
                }
            }
        }

        // GC% of actual bases (excluding gaps/N)
        let gc_pct = gc_count as f64 / (total_nucleotides - missing_count) as f64 * 100.0;
        let missing_pct = missing_count as f64 / total_nucleotides as f64 * 100.0;

        // check if its an alignment, in order to know if we should calc variable and informative site/pct stats
        let all_equal = sequences.values().all(|s| s.len() == length);

        // strip path off of the filename
        let filename = Path::new(&file).file_name().unwrap().to_str().unwrap();

        if all_equal {
            let (variable, informative) = count_informative_sites(&sequences, length);
            let variable_pct = variable as f64 / length as f64 * 100.0;
            let informative_pct = informative as f64 / length as f64 * 100.0;
            println!(
                "{}\t{}\t{}\t{:.1}\t{:.1}\t{}\t{:.1}\t{}\t{:.1}",
                filename,
                num_sequences,
                length,
                gc_pct,
                missing_pct,
                variable,
                variable_pct,
                informative,
                informative_pct
            );
        } else {
            println!(
                "{}\t{}\tNA\t{:.1}\t{:.1}\tNA\tNA\tNA\tNA",
                filename, num_sequences, gc_pct, missing_pct
            );
        }
    }
}
