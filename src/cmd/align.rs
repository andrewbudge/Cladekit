use std::fs::{File, OpenOptions};
use std::path::Path;
use std::process::Command;

use clap::Args;

#[derive(Args)]
pub struct AlignArgs {
    /// Alignment program name (only Mafft currently, Muscle and others planned for in the future.)
    #[arg(short, long)]
    pub program: String,

    /// Input unaligned FASTA files
    #[arg(short, long, required = true, num_args = 1..)]
    pub input: Vec<String>,

    /// Suffix to append to aligned output filenames (e.g., _aln)
    #[arg(short, long, default_value = "_aln")]
    pub extension: String,

    /// Output directory for aligned files
    #[arg(short, long)]
    pub output: String,

    /// Additional arguments passed directly to the alignment program
    #[arg(last = true)]
    pub passthrough: Vec<String>,
}

pub fn run(args: AlignArgs) {
    // Check if program exists / is callable
    match Command::new(&args.program).arg("--version").output() {
        Ok(_) => {}
        Err(_) => {
            eprintln!(
                "Error: '{}' not found. Make sure it is installed and in your PATH.",
                args.program
            );
            std::process::exit(1);
        }
    }

    std::fs::create_dir_all(&args.output).expect("Could not create output directory");

    // Create a log file for aligner stderr output
    let log_path = Path::new(&args.output).join("align.log");
    File::create(&log_path).expect("Could not create log file");

    for input_path in &args.input {
        let path = Path::new(input_path);
        let stem = path
            .file_stem()
            .expect("Could not get filename")
            .to_str()
            .unwrap();
        let ext = path
            .extension()
            .map(|e| e.to_str().unwrap())
            .unwrap_or("fasta");
        let output_filename = format!("{}{}.{}", stem, args.extension, ext);
        let output_path = Path::new(&args.output).join(&output_filename);

        eprint!("Aligning {}...", stem);

        let output_file = File::create(&output_path).expect("Could not create output file");

        let log_file = OpenOptions::new()
            .append(true)
            .open(&log_path)
            .expect("Could not open log file");

        // Use --auto by default
        let mut cmd = Command::new(&args.program);
        if args.passthrough.is_empty() {
            cmd.arg("--auto");
        } else {
            cmd.args(&args.passthrough);
        }

        let status = cmd
            .arg(input_path)
            .stdout(output_file) // capture alignment
            .stderr(log_file) // capture log
            .status()
            .expect("Failed to run aligner");

        if !status.success() {
            eprintln!("FAILED (see {})", log_path.display());
            std::process::exit(1);
        }
        eprintln!("done");
    }

    eprintln!("Done. Aligned {} files.", args.input.len());
}
