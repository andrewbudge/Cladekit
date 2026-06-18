use serde::{Deserialize, Serialize};

/// A single nucleotide record returned by `query`, populated from an NCBI
/// esummary docsum. No homology/locus information is present at this stage —
/// that is determined later by MMseqs2 in `extract`.
#[derive(Serialize, Deserialize, Debug)]
pub struct Accession {
    pub accession: String,
    pub taxon_name: String,
    /// The record's own source-organism TaxID (from the docsum), e.g. a single
    /// species. Distinct from [`Accession::query_taxid`].
    pub taxid: u64,
    pub length: usize,
    /// GenBank title string (e.g. "...cytochrome c oxidase subunit I..."). Recorded
    /// for traceability only; deliberately NOT used as a homology/quality filter.
    pub gene_annotation: String,
    pub refseq: bool,
    pub source_db: String,
    /// The higher-level TaxID that was queried to surface this record (the
    /// ingroup/outgroup root the user supplied). Stamped here so provenance
    /// survives flattening, and so the lineage check has the root to compare
    /// [`Accession::taxid`] against.
    pub query_taxid: u64,
    /// Which group the querying root belongs to. Stamped per-record (rather than
    /// only on the enclosing [`QueryResult`]) so it travels with the accession
    /// once the array is flattened downstream.
    pub taxon_group: TaxonGroup,
    /// Flagged (never silently dropped) when a record's source TaxID does not fall
    /// within the queried taxonomic root. Always `false` until the lineage check is
    /// implemented.
    pub taxonomic_outlier: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaxonGroup {
    Ingroup,
    Outgroup,
}

/// The result of one esearch over a single taxon. `query_results.json` is a
/// JSON array of these — one element per queried taxon, across both the ingroup
/// and outgroup TaxIDs given on the command line.
#[derive(Serialize, Deserialize, Debug)]
pub struct QueryResult {
    pub taxid: u64,
    pub taxon_name: String,
    pub taxon_group: TaxonGroup,
    pub total_accessions: usize,
    pub accessions: Vec<Accession>,
}
