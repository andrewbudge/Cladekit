<div align="left">
  <img src="docs/mockup_logos/cladekit-slickrock-horizontal-dark.svg" width = "600">
</div>

Cladekit is a lightweight, composable CLI phylogenetics toolkit. A single binary, Cladekit provides many subcommands that replace common chains of bash commands or a collection of individual programs in phylogenetic pipelines. Examples include header extraction, concatenation, and alignment quality control.

**Note:** Cladekit is under active development. Subcommands may change or be added as the project matures.

## Install

Requires [Rust](https://www.rust-lang.org/tools/install).

```bash
cargo install --git https://github.com/andrewbudge/cladekit
```

This builds the binary and adds `cladekit` to your PATH.

To update to the latest version:

```bash
cargo install --force --git https://github.com/andrewbudge/cladekit
```

## Subcommands
---
### getheaders (ghd)

Extract headers from FASTA files.

**Example:**

```bash
$ cladekit getheaders testdata/test_good.fasta
Sequence1
Sequence2
Sequence1

$ cladekit getheaders -u testdata/test_good.fasta
Sequence1
Sequence2
```
---
### concat (liger)

Concatenate multiple gene alignments into a supermatrix. Unlike other tools, input files can live anywhere and globs are accepted.

Concat runs in two modes:

- **Exact match (default):** headers must match exactly across files, like FASconCAT and AMAS.
- **Smart match (`-a alias.txt`):** pass an alias list — a file of clean output names (one per line, e.g. `Mus_musculus`) that get matched to messy input headers via case-insensitive substring search. Longer aliases match first to prevent partial collisions (e.g. `Mus musculus domesticus` claims before `Mus musculus`). Once a header is claimed it cannot be matched again. The alias list doubles as a rename map — input headers stay messy, output gets clean names. Requires `-l` for a provenance TSV that records exactly which original header matched each alias.

Concat auto-detects DNA vs amino acid data per gene and adjusts missing characters and partition labels accordingly. FASTA output goes to stdout, partition boundaries to stderr in RAxML/IQ-TREE format by default. NEXUS bundles everything into one file.

**Exact match — clean headers:**

```bash
$ cladekit concat gene1.fasta gene2.fasta > supermatrix.fasta
DNA, gene1.fasta = 1-4
DNA, gene2.fasta = 5-8
```

**Smart match — messy headers with an alias list:**

```bash
$ cat alias.txt
Mus_musculus
Rattus_rattus
Xenopus_laevis

$ cladekit concat -a alias.txt -l prov.tsv gene1.fasta gene2.fasta > supermatrix.fasta
DNA, gene1.fasta = 1-4
DNA, gene2.fasta = 5-8

$ cat supermatrix.fasta
>Mus_musculus
ATCGATCG
>Rattus_rattus
ATCGNNNN
>Xenopus_laevis
NNNNATCG

$ cat prov.tsv
alias.txt	gene1.fasta	gene2.fasta
Mus_musculus	AB123.1 Mus musculus gene1 cds	XM456.1 Mus musculus gene2 cds
Rattus_rattus	AB124.1 Rattus rattus gene1 cds	MISSING
Xenopus_laevis	MISSING	XM789.1 Xenopus laevis gene2 cds
```

**NEXUS output:**

```bash
$ cladekit concat -a alias.txt -l prov.tsv -f nexus gene1.fasta gene2.fasta
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
- `-a, --alias` — alias list for smart matching (clean output names that map to messy input headers)
- `-l, --log` — provenance TSV output file (required with `-a`)
- `-f, --format` — output format: fasta (default), nexus (also accepts `n` or `nex`)
- `-m, --missing` — override missing data character (default: auto per data type — N for DNA, X for amino acid, ? for mixed)
- `-p, --partitions` — partition format: raxml (default, also used by IQ-TREE) or nexus
- `--dry-run` — show a matching summary (per-gene match counts and per-taxon coverage) without building the supermatrix
---
### stats

Get basic alignment statistics from FASTA files. Accepts multiple files via globs. Automatically detects DNA vs amino acid sequences.

**Columns:**
- **file** — filename (path stripped)
- **sequences** — number of sequences
- **length** — alignment length (NA if unaligned)
- **type** — `DNA` or `AA` (auto-detected, supports IUPAC ambiguity codes)
- **gc_pct** — GC content as a percentage of real bases (NA for amino acid data)
- **missing_pct** — percentage of gaps and unknown characters
- **variable** — sites with at least 2 different residues (excluding gaps/unknowns)
- **variable_pct** — variable sites as a percentage of alignment length
- **informative** — parsimony-informative sites (at least 2 residues each appearing 2+ times)
- **informative_pct** — informative sites as a percentage of alignment length

**Example:**

```bash
$ cladekit stats supermatrix.fasta proteins.fasta
file	sequences	length	type	gc_pct	missing_pct	variable	variable_pct	informative	informative_pct
supermatrix.fasta	3	8	DNA	50.0	33.3	0	0.0	0	0.0
proteins.fasta	4	20	AA	NA	0.0	3	15.0	2	10.0
```

**Flags:**
- `-d, --detailed` — per-sequence statistics (header, length, GC%, missingness)
- `-p, --pretty` — column-aligned output for readability
---
### coverage

Summarize taxa and loci coverage from a concat provenance TSV. Shows how many loci each taxon appears in, or how many taxa each locus has.

**Example:**

```bash
$ cladekit coverage -t prov.tsv
taxa	loci_present	loci_missing	pct_missing
Mus_musculus	5/5	0/5	0.0%
Smilodon_populator	2/5	3/5	60.0%

$ cladekit coverage -l -p prov.tsv
loci          appearance_count  missing_pct
12S_aln.fas   6/8               25.0%
COX1_aln.fas  6/8               25.0%
```

**Flags:**
- `-t, --taxa` — show per-taxon coverage (how many loci each taxon has)
- `-l, --loci` — show per-loci coverage (how many taxa each locus has)
- `-p, --pretty` — column-aligned output for readability
---
### convert

Convert between common sequence data file types. Auto-detects the input format from file contents.

**Supported formats:**
- FASTA (`f`)
- NEXUS (`n` / `nex` / `nexus`)
- Relaxed PHYLIP (`rp` / `phylip`)
- Strict PHYLIP (`sp`)

**Example:**

```bash
$ cladekit convert -o n alignment.fasta
#NEXUS
BEGIN DATA;
  DIMENSIONS NTAX=3 NCHAR=8;
  FORMAT DATATYPE=DNA MISSING=N GAP=-;
  MATRIX
  Taxon_A    ATCGATCG
  Taxon_B    ATCGATCG
  Taxon_C    ATCGNNNN
;
END;

$ cladekit convert -o rp alignment.nex
3 8
Taxon_A    ATCGATCG
Taxon_B    ATCGATCG
Taxon_C    ATCGNNNN
```

**Flags:**
- `-o, --output_format` — output format: `f` (fasta), `n` (nexus), `rp` (relaxed phylip), `sp` (strict phylip)
---
### extract

Extract gene regions from target organism sequences using homology search. Takes a reference FASTA with labeled gene sequences and one or more target FASTAs (or a directory), runs MMseqs2 `easy-search`, and writes one output FASTA per gene containing the extracted region from each organism that had a hit.

Requires [MMseqs2](https://github.com/soedinglab/MMseqs2) installed and in your PATH.

**Example:**

```bash
# reference.fasta has labeled genes: >COX1, >ND2, >12S
# targets/ contains one FASTA per organism

$ cladekit extract -r reference.fasta -t targets/ -o genes/
Pooling 12 target files...
Running MMseqs2 easy-search...
Parsing results...
Done. Extracted 3 gene(s) from 34 hits.

$ ls genes/
COX1.fasta  ND2.fasta  12S.fasta
```

**Flags:**
- `-r, --reference` — reference FASTA with labeled gene sequences (e.g., `>COX1`, `>ND2`)
- `-t, --targets` — target organism FASTA files or a directory containing them
- `-o, --output` — output directory for per-gene FASTAs
- `-s, --sensitivity` — MMseqs2 sensitivity, 1.0 (fast) to 7.5 (max); default 5.7
- `--min-coverage` — minimum fraction of the reference gene that must be covered to keep a hit (default: 0.5)
- `--flank` — extra bases to grab on either side of each hit (default: 0)
- `--keep-intermediates` — keep the temp directory with pooled targets and raw MMseqs2 output
---
### align (aln)

Batch align multiple FASTA files using an external alignment program. Runs the aligner on each input file and writes output to a directory with a consistent naming convention.

**Example:**

```bash
$ cladekit align -p mafft -i genes/*.fasta -e _aln -o aligned/
Aligning COI...done
Aligning ND2...done
Aligning 12S...done
Done. Aligned 3 files.
```

Pass custom flags to the aligner after `--`:

```bash
$ cladekit align -p mafft -i genes/*.fasta -e _aln -o aligned/ -- --thread 4 --maxiterate 1000
```

**Flags:**
- `-p, --program` — alignment program name or path (e.g., `mafft`, `/usr/local/bin/muscle`)
- `-i, --input` — input unaligned FASTA files (glob or list)
- `-e, --extension` — suffix to append to output filenames (default: `_aln`)
- `-o, --output` — output directory for aligned files
- `--` — everything after `--` is passed through to the aligner verbatim
---
### filter

Remove taxa from an alignment that exceed a missingness threshold, have too few loci in a supermatrix, or both. Filters can be used independently or combined — a taxon must pass all applied filters to be kept. Output goes to stdout, summary to stderr.

**Example:**

```bash
# drop taxa with more than 50% gaps in the supermatrix
$ cladekit filter supermatrix.fasta --max-missing 0.5 > filtered.fasta
Total taxa: 8
Kept taxa: 6
Dropped taxa: 2

# drop taxa present in fewer than 3 loci (requires coverage TSV from cladekit coverage)
$ cladekit coverage -t prov.tsv > coverage.tsv
$ cladekit filter supermatrix.fasta --min-loci 3 -l coverage.tsv > filtered.fasta

# both filters at once
$ cladekit filter supermatrix.fasta --max-missing 0.5 --min-loci 3 -l coverage.tsv > filtered.fasta
```

**Flags:**
- `--max-missing` — maximum allowed missingness fraction per taxon (0.0–1.0)
- `--min-loci` — minimum number of loci a taxon must be present in
- `-l, --log` — coverage TSV from `cladekit coverage` (required with `--min-loci`)
---
### curate

Trim alignment columns by parsimony-informativeness and gappiness. A native Rust port of [ClipKIT](https://journals.plos.org/plosbiology/article?id=10.1371/journal.pbio.3001007) (Steenwyk et al. 2020). Accepts multiple files and a glob. Output goes to a directory or stdout, summary to stderr.

Compose keep conditions with `-k`: `p` = parsimony-informative, `c` = constant, `g` = gappiness filter. Combine letters for mixed modes.

**Example:**

```bash
# single file to stdout
$ cladekit curate alignment.fasta > trimmed.fasta
Sites in:      10000
Sites kept:    4321
Sites removed: 5679

# batch with glob, output to directory
$ cladekit curate aligned/*.fasta -o curated/
COI: 8000 → 3201 sites (4799 removed)
ND2: 6000 → 2874 sites (3126 removed)
12S: 4000 → 1950 sites (2050 removed)
Done. Curated 3 files.

# custom extension and stricter gap threshold
$ cladekit curate aligned/*.fasta -e _trim --gap-threshold 0.5 -o curated/
```

**Flags:**
- `-k, --keep` — column properties to keep: `p` (parsimony-informative), `c` (constant), `g` (gappiness filter). Combine letters: `p`, `pc`, `pg` (default), `pcg`, `g`
- `--gap-threshold` — maximum allowed gappiness per column (0.0–1.0), used when `g` is in `--keep` (default: 0.9)
- `-e, --extension` — suffix to append to output filenames (default: `_curated`)
- `-o, --output` — output directory for trimmed files (if omitted, writes to stdout)

**Mode reference:**

| Flag | ClipKIT equivalent |
|---|---|
| `-k p` | `kpi` |
| `-k pc` | `kpic` |
| `-k g` | `gappy` |
| `-k pg` | `kpi-gappy` (default) |
| `-k pcg` | `kpic-gappy` |

---
## Planned Subcommands
- **scrub** — alignment outlier detection via pairwise p-distances
- **drafttree** — quick neighbor-joining tree from an MSA for sanity-checking alignments before committing to ML/Bayesian methods
- **view** — in-terminal alignment viewer
- **slice** — cut out or extract sections of an alignment

## Development Note

Cladekit is being built as both a real research tool and a vehicle for learning Rust. Development is assisted by Claude (Anthropic), which serves as a teaching aid and coding partner. The design, domain knowledge, and direction are the author's own.

## Author

Andrew Budge
