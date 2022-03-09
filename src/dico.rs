use std::io::{BufReader, BufRead};

pub enum DicoError {
    InvalidChar(usize, char),
    NoFile(char),
    ReadingFile,
}

/// return the dico plus id of best word if provided by the dico
pub fn load(dico: char, word_len: u8) -> Result<(Vec<String>, Option<usize>), DicoError> {
    let mut dico = BufReader::new(
        std::fs::File::open(format!("dico/{}.txt", dico)).map_err(|_| DicoError::NoFile(dico))?
    ).lines().enumerate();

    let mut best = None;
    let word_len = word_len as usize;
    let mut matchs = Vec::with_capacity(150);

    for _ in 0..4 {
        if let Some((line, row)) = dico.next() {
            if line == word_len - 6 {
                if let Ok(row) = row {
                    if let Ok(id) = row.parse() {
                        best = Some(id);
                    }
                }
            }
        }
    }

    for (line, row) in dico {
        match row {
            Ok(row) => if word_len == row.len() {
                is_valid_word(&row).map_err(|c| DicoError::InvalidChar(line + 1, c))?;
                matchs.push(row);
            }
            Err(_) => return Err(DicoError::ReadingFile),
        }
    }

    Ok((matchs, best))
}

fn is_valid_word(word: &str) -> Result<(), char> {
    if let Some(invalid) = word.chars().find(|c| !('a'..='z').contains(c)) {
        Err(invalid)
    } else {
        Ok(())
    }
}

impl std::fmt::Display for DicoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DicoError::InvalidChar(line, char) => write!(f, "Error on line {}: invalid char '{}'", line, char),
            DicoError::NoFile(file) => write!(f, "No file named dico/{}.txt", file),
            DicoError::ReadingFile => write!(f, "Error while reading file!"),
        }
    }
}
