use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct CleanArgs {
    /// Directory of per-gene FASTAs written by `extract`
    #[arg(long)]
    pub genes_dir: PathBuf,

    /// Path to query_results.json (the accession -> TaxID/Name join table)
    #[arg(long, short = 'q')]
    pub query: PathBuf,

    /// Enable NUMT detection via reading-frame check
    #[arg(long)]
    pub check_reading_frame: bool,

    /// NCBI genetic code table number (2 = vertebrate mitochondrial)
    #[arg(long, default_value = "2")]
    pub genetic_code: u8,

    /// Output directory
    #[arg(long, short = 'o')]
    pub out: PathBuf,
}

pub async fn run(_args: CleanArgs) -> anyhow::Result<()> {
    todo!("clean: rewrite headers to TaxID|Name|Accession|Gene, NUMT check, per-gene dedup")
}
