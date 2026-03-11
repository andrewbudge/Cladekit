use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

// Function to parse fasta (incredible naming)
// Takes in a fasta and makes it into a hashmap to be used by the subcommands
pub fn parse_fasta(
    filename: &str,
    validate_equal: bool,
) -> Result<(HashMap<String, String>, usize), String> {
    let file = File::open(filename).map_err(|e| format!("Could not open {}: {}", filename, e))?;

    // file reader
    let reader = BufReader::new(file);

    // Main map and then vars to track seqs as we read
    let mut sequences = HashMap::new();
    let mut current_header = String::new();
    let mut current_seq = String::new();
    let mut expected_length: Option<usize> = None;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Error reading file: {}", e))?;
        let line = line.trim().to_string();

        if line.starts_with('>') {
            if !current_header.is_empty() {
                // Uppercase seq and insert header and seq to hashmap
                current_seq = current_seq.to_uppercase();
                sequences.insert(current_header.clone(), current_seq.clone());
                // Validate seq length (make sure its only passing aligned files in)
                if validate_equal {
                    match expected_length {
                        None => expected_length = Some(current_seq.len()),
                        Some(len) => {
                            if current_seq.len() != len {
                                return Err(format!(
                                    "Error: Sequence lenght mismatch in {} : {}",
                                    filename, current_header
                                ));
                            }
                        }
                    }
                }
            }
            // start new header
            current_header = line[1..].to_string();
            current_seq.clear();
        } else if !line.is_empty() {
            current_seq.push_str(&line);
        }
    }
    if !current_header.is_empty() {
        current_seq = current_seq.to_uppercase();
        sequences.insert(current_header, current_seq);
    }

    let length = sequences.values().next().map_or(0, |s| s.len());
    Ok((sequences, length))
}

// Function to read in taxa list
// Taxa list must be one name per line
pub fn load_taxa_list(filename: &str) -> Result<Vec<String>, String> {
    // open file and create reader
    let file = File::open(filename).map_err(|e| format!("Could not open {}: {}", filename, e))?;
    let reader = BufReader::new(file);

    // var to store taxa
    let mut taxa = Vec::new();

    // loop through ever line in the list, trim and turn into string, then add to vector
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Error reading file: {}", e))?;
        let line = line.trim().to_string();
        if !line.is_empty() {
            taxa.push(line);
        }
    }

    Ok(taxa)
}
