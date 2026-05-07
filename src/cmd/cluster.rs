use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::Command;

use clap::Args;

#[derive(Args)]
pub struct ClusterArgs {
    /// Input FASTA files (organism files to cluster)
    #[arg(short, long, required = true, num_args = 1..)]
    pub input: Vec<String>,

    /// Output directory for per-cluster FASTA files
    #[arg(short, long)]
    pub output: String,

    /// Clustering identity threshold (0.0-1.0)
    #[arg(long, default_value = "0.95")]
    pub identity: f64,

    /// Use fast linear-time clustering (easy-linclust) instead of sensitive (easy-cluster)
    #[arg(long)]
    pub fast: bool,

    /// Keep intermediate MMseqs2 files for debugging
    #[arg(long)]
    pub keep_intermediates: bool,
}

pub fn run(args: ClusterArgs) {
    // 1. Check that mmseqs is installed
    match Command::new("mmseqs").arg("version").output() {
        Ok(_) => {}
        Err(_) => {
            eprintln!("Error: 'mmseqs' not found. Make sure MMseqs2 is installed and in your PATH.");
            std::process::exit(1);
        }
    }

    // 2. Create output directory
    fs::create_dir_all(&args.output).expect("Could not create output directory");

    // 3. Concatenate all input FASTAs into one temp file for MMseqs2
    let tmp_dir = Path::new(&args.output).join(".cluster_tmp");
    fs::create_dir_all(&tmp_dir).expect("Could not create temp directory");

    let combined_path = tmp_dir.join("combined.fasta");
    {
        let mut combined = File::create(&combined_path).expect("Could not create combined FASTA");
        for input_path in &args.input {
            let data = fs::read(input_path)
                .unwrap_or_else(|_| panic!("Could not read {}", input_path));
            combined.write_all(&data).expect("Could not write to combined FASTA");
        }
    }

    // 4. Run MMseqs2 easy-cluster or easy-linclust
    let cluster_mode = if args.fast { "easy-linclust" } else { "easy-cluster" };
    let result_prefix = tmp_dir.join("result");

    eprint!("Clustering with {}...", cluster_mode);

    let status = Command::new("mmseqs")
        .arg(cluster_mode)
        .arg(&combined_path)
        .arg(&result_prefix)
        .arg(&tmp_dir)
        .arg("--min-seq-id")
        .arg(args.identity.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("Failed to run mmseqs");

    if !status.success() {
        eprintln!("FAILED");
        eprintln!("MMseqs2 clustering failed. Run with --keep-intermediates to debug.");
        std::process::exit(1);
    }
    eprintln!("done");

    // 5. Parse the cluster TSV output and group sequences
    // MMseqs2 easy-cluster outputs: result_cluster.tsv (rep_seq \t member_seq)
    let cluster_tsv = format!("{}_cluster.tsv", result_prefix.display());
    let tsv_file = File::open(&cluster_tsv)
        .unwrap_or_else(|_| panic!("Could not open cluster results: {}", cluster_tsv));

    // Map: representative -> list of member sequence headers
    let mut clusters: HashMap<String, Vec<String>> = HashMap::new();
    for line in BufReader::new(tsv_file).lines() {
        let line = line.expect("Could not read cluster TSV");
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            clusters.entry(parts[0].to_string())
                .or_default()
                .push(parts[1].to_string());
        }
    }

    // 6. Read all sequences from the combined FASTA into memory
    let fasta_file = File::open(&combined_path).expect("Could not reopen combined FASTA");
    let mut sequences: HashMap<String, String> = HashMap::new();
    let mut current_header = String::new();
    let mut current_seq = String::new();

    for line in BufReader::new(fasta_file).lines() {
        let line = line.expect("Could not read FASTA line");
        if line.starts_with('>') {
            if !current_header.is_empty() {
                sequences.insert(current_header.clone(), current_seq.clone());
            }
            current_header = line[1..].to_string();
            current_seq.clear();
        } else {
            current_seq.push_str(line.trim());
        }
    }
    if !current_header.is_empty() {
        sequences.insert(current_header, current_seq);
    }

    // 7. Write per-cluster FASTA files
    eprint!("Writing {} clusters...", clusters.len());
    for (i, (_rep, members)) in clusters.iter().enumerate() {
        let cluster_path = Path::new(&args.output).join(format!("cluster_{}.fasta", i + 1));
        let mut out = File::create(&cluster_path).expect("Could not create cluster file");

        for member in members {
            if let Some(seq) = sequences.get(member) {
                writeln!(out, ">{}", member).unwrap();
                writeln!(out, "{}", seq).unwrap();
            }
        }
    }
    eprintln!("done");

    // 8. Clean up temp files unless --keep-intermediates
    if !args.keep_intermediates {
        fs::remove_dir_all(&tmp_dir).ok();
    } else {
        eprintln!("Intermediate files kept at: {}", tmp_dir.display());
    }

    eprintln!("Done. {} clusters written to {}", clusters.len(), args.output);
}
