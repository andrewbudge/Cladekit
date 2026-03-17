# Phylo

Phylo is a lightweight, composable CLI phylogenetics toolkit. A single binary, Phylo provides many subcommands that replace common chains of bash commands or a collection of individual programs in phylogenetic pipelines. Examples include header extraction, concatenation, and alignment quality control.

**Note:** Phylo is under active development. Subcommands may change or be added as the project matures.

## Install

Requires [Rust](https://www.rust-lang.org/tools/install).

```bash
cargo install --git https://github.com/andrewbudge/phylo
```

This builds the binary and adds `phylo` to your PATH.

To update to the latest version:

```bash
cargo install --force --git https://github.com/andrewbudge/phylo
```

## Subcommands

### getheaders (ghd)

Extract headers from FASTA files.

**Example:**

```bash
$ phylo getheaders testdata/test_good.fasta
Sequence1
Sequence2
Sequence1

$ phylo getheaders -u testdata/test_good.fasta
Sequence1
Sequence2
```

### concat (liger)

Concatenate multiple gene alignments into a supermatrix. Over **1,000x faster** than FASconCAT-G on a 100 taxa x 20 gene benchmark (9ms vs 9.8s), and unlike other tools, input files can live anywhere — no need to dump everything into one directory.

Concat runs in two modes:

- **Exact match (default):** headers must match exactly across files, like FASconCAT and AMAS. Simple and fast.
- **Smart match (`-t taxa.txt`):** case-insensitive substring matching links a taxa list to messy GenBank-style headers. Longer taxon names match first to prevent partial collisions (e.g., "Mus musculus domesticus" claims before "Mus musculus"). Requires `-l` for a provenance TSV — the audit trail that records which original header matched each taxon.

FASTA output goes to stdout, partition boundaries to stderr. NEXUS bundles everything into one file.

**Exact match — clean headers:**

```bash
$ phylo concat gene1.fasta gene2.fasta > supermatrix.fasta
gene1.fasta = 1-4
gene2.fasta = 5-8
```

**Smart match — messy headers:**

```bash
$ cat taxa.txt
Mus_musculus
Rattus_rattus
Xenopus_laevis

$ phylo concat -t taxa.txt -l prov.tsv gene1.fasta gene2.fasta > supermatrix.fasta
gene1.fasta = 1-4
gene2.fasta = 5-8

$ cat supermatrix.fasta
>Mus_musculus
ATCGATCG
>Rattus_rattus
ATCGNNNN
>Xenopus_laevis
NNNNATCG

$ cat prov.tsv
taxa.txt	gene1.fasta	gene2.fasta
Mus_musculus	AB123.1 Mus musculus gene1 cds	XM456.1 Mus musculus gene2 cds
Rattus_rattus	AB124.1 Rattus rattus gene1 cds	MISSING
Xenopus_laevis	MISSING	XM789.1 Xenopus laevis gene2 cds
```

**NEXUS output:**

```bash
$ phylo concat -t taxa.txt -l prov.tsv -f nexus gene1.fasta gene2.fasta
#NEXUS
BEGIN DATA;
  DIMENSIONS NTAX=3 NCHAR=8;
  FORMAT DATATYPE=DNA MISSING=N GAP=-;
  MATRIX
  Mus_musculus    ATCGATCG
  Rattus_rattus   ATCGNNNN
  Xenopus_laevis  NNNNATCG
;
END;
BEGIN SETS;
  CHARSET gene1.fasta = 1-4;
  CHARSET gene2.fasta = 5-8;
END;
```

**Flags:**
- `-t, --taxa` — taxa list for smart matching (enables substring search + header renaming)
- `-l, --log` — provenance TSV output file (required with `-t`)
- `-f, --format` — output format: fasta (default), nexus (also accepts `n` or `nex`)
- `-m, --missing` — character for missing data (default: N)

### stats

Get basic alignment statistics from FASTA files. Accepts multiple files via globs.

**Columns:**
- **file** — filename (path stripped)
- **sequences** — number of sequences
- **length** — alignment length (NA if unaligned)
- **gc_pct** — GC content as a percentage of real bases (excluding gaps/N)
- **missing_pct** — percentage of gaps and N characters
- **variable** — sites with at least 2 different bases (excluding gaps/N)
- **variable_pct** — variable sites as a percentage of alignment length
- **informative** — parsimony-informative sites (at least 2 bases each appearing 2+ times)
- **informative_pct** — informative sites as a percentage of alignment length

**Example:**

```bash
$ cat supermatrix.fasta
>Mus_musculus
ATCGATCG
>Rattus_rattus
ATCGNNNN
>Xenopus_laevis
NNNNATCG

$ phylo stats supermatrix.fasta
file	sequences	length	gc_pct	missing_pct	variable	variable_pct	informative	informative_pct
supermatrix.fasta	3	8	50.0	33.3	0	0.0	0	0.0
```

## Planned Subcommands

- **coverage** — taxa coverage across gene files
- **scrub** — alignment outlier detection via pairwise p-distances
- **view** - in terminal alignment viewer
- **slice** - cut out and remove sections of an alignment (remove non-homologous seqs, extract homologous seqs)
- **convert** - convert between sequence data file types (FASTA, Nexus, Relaxed and Strict Phylip)

## Development Note

Phylo is being built as both a real research tool and a vehicle for learning Rust. Development is assisted by Claude (Anthropic), which serves as a teaching aid and coding partner. The design, domain knowledge, and direction are the author's own.

## Author

Andrew Budge
