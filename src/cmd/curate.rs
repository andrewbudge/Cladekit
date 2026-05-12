use cladekit::parse_fasta;
use clap::Args;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Args)]
pub struct CurateArgs {
    /// Aligned FASTA files to trim
    #[arg(required = true, num_args = 1..)]
    pub input: Vec<String>,

    /// Which column properties to keep: p = parsimony-informative, c = constant, g = gappiness filter
    /// Combine letters: "pg" (default, smart-gap), "p", "pc", "pcg", "g"
    #[arg(short, long, default_value = "pg")]
    pub keep: String,

    /// Maximum allowed gappiness per site (0.0–1.0), used when 'g' is in --keep
    #[arg(long, default_value_t = 0.9)]
    pub gap_threshold: f64,

    /// Suffix to append to output filenames (e.g., _curated)
    #[arg(short, long, default_value = "_curated")]
    pub extension: String,

    /// Output directory for trimmed files (if omitted, writes to stdout)
    #[arg(short, long)]
    pub output: Option<String>,
}

pub fn run(args: CurateArgs) {
    if let Some(ref out_dir) = args.output {
        std::fs::create_dir_all(out_dir).expect("Could not create output directory");
    }

    let total_files = args.input.len();

    for input_path in &args.input {
        let (sequences, _) = parse_fasta(input_path, true)
            .expect("Could not read alignment (file not found or sequences are not the same length)");

        let seqs: Vec<(String, Vec<u8>)> = sequences
            .into_iter()
            .map(|(h, s)| (h, s.into_bytes()))
            .collect();

        let n_taxa = seqs.len();
        let aln_len = seqs[0].1.len();

        let keep_p = args.keep.contains('p');
        let keep_c = args.keep.contains('c');
        let keep_g = args.keep.contains('g');

        let mut kept_cols: Vec<usize> = Vec::new();

        for col in 0..aln_len {
            let mut gap_count = 0usize;
            let mut freq = std::collections::HashMap::<u8, usize>::new();

            for (_, seq) in &seqs {
                let base = seq[col];
                if base == b'-' {
                    gap_count += 1;
                } else if base != b'N' && base != b'?' && base != b'X' {
                    *freq.entry(base).or_insert(0) += 1;
                }
            }

            let gappiness = gap_count as f64 / n_taxa as f64;

            let qualifying: usize = freq.values().filter(|&&c| c >= 2).count();
            let is_parsimony_informative = qualifying >= 2;

            let is_constant = freq.len() == 1 && *freq.values().next().unwrap() >= 2;

            let keep = match (keep_p, keep_c, keep_g) {
                (true, false, true) => is_parsimony_informative && gappiness < args.gap_threshold,
                (true, true, true) => {
                    (is_parsimony_informative || is_constant) && gappiness < args.gap_threshold
                }
                (true, true, false) => is_parsimony_informative || is_constant,
                (true, false, false) => is_parsimony_informative,
                (false, false, true) => gappiness < args.gap_threshold,
                _ => is_parsimony_informative && gappiness < args.gap_threshold,
            };

            if keep {
                kept_cols.push(col);
            }
        }

        let kept = kept_cols.len();
        let path = Path::new(input_path);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("fasta");

        if let Some(ref out_dir) = args.output {
            let out_filename = format!("{}{}.{}", stem, args.extension, ext);
            let out_path = Path::new(out_dir).join(&out_filename);
            let file = File::create(&out_path).expect("Could not create output file");
            let mut writer = BufWriter::new(file);

            for (header, seq) in &seqs {
                let trimmed: String = kept_cols.iter().map(|&col| seq[col] as char).collect();
                writeln!(writer, ">{}", header).unwrap();
                writeln!(writer, "{}", trimmed).unwrap();
            }

            eprintln!("{}: {} → {} sites ({} removed)", stem, aln_len, kept, aln_len - kept);
        } else {
            for (header, seq) in &seqs {
                let trimmed: String = kept_cols.iter().map(|&col| seq[col] as char).collect();
                println!(">{}", header);
                println!("{}", trimmed);
            }

            eprintln!("Sites in:      {}", aln_len);
            eprintln!("Sites kept:    {}", kept);
            eprintln!("Sites removed: {}", aln_len - kept);
        }
    }

    if args.output.is_some() && total_files > 1 {
        eprintln!("Done. Curated {} files.", total_files);
    }
}
