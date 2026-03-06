use clap::Args;

#[derive(Args)]
pub struct GetheadersArgs {
    /// Read in fasta, read from stdin if no file
    pub file: Option<String>,

    /// unique headers flag
    #[arg(short,long)]
    pub unique: bool,
}

pub fn run(args: GetheadersArgs) {
    println!("file: {:?}, unique : {}", args.file, args.unique);
}
