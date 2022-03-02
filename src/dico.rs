use std::io::{BufReader, BufRead, Write};
use crate::word::{Word, CharError};

const DICO_NAME: &str = "i_dico.txt";

pub enum DicoError<'a> {
    InvalidMask(&'a str),
    Inner(CharError),
    ReadingFile,
    NoFile,
}

pub fn load(mask: &str) -> Result<Vec<Word>, DicoError> {
    if !is_valid_mask(mask) {
        return Err(DicoError::InvalidMask(mask));
    }

    let dico = BufReader::new(
        std::fs::File::open(DICO_NAME).map_err(|_| DicoError::NoFile)?
    ).lines();

    let mask_len = mask.len();
    let mut matchs = Vec::with_capacity(150);

    for row in dico {
        match row {
            Ok(row) => if mask_len == row.len() && mask_match(mask, &row) {
                matchs.push(Word::new(row)?);
            }
            Err(_) => return Err(DicoError::ReadingFile),
        }
    }

    Ok(matchs)
}

pub fn add(word: &str) {
    if !is_valid_word(word) {
        eprintln!("'{}' is not a valid word.", word);
        return;
    }

    let file = match std::fs::File::open(DICO_NAME) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let dico = BufReader::new(file).lines();
    let word_len = word.len();

    for row in dico {
        match row {
            Ok(row) => if word_len == row.len() && mask_match(word, &row) {
                eprintln!("the word '{}' already exist in the dico", word);
                return;
            }
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        }
    }

    let r_file = std::fs::File::options()
        .append(true)
        .write(true)
        .open(DICO_NAME);

    let mut file = match r_file {
        Ok(file) => file,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    file.write_all(word.as_bytes()).unwrap();
    file.write(b"\n").unwrap();
}

fn is_valid_mask(mask: &str) -> bool {
    mask.chars().all(|c| (c >= 'a' && c <= 'z') || c == '.')
}

fn is_valid_word(word: &str) -> bool {
    word.chars().all(|c| c >= 'a' && c <= 'z')
}

// assume mask and target has the same size.
fn mask_match(mask: &str, target: &str) -> bool {
    mask.chars().zip(target.chars()).all(|(mask, target)| {
        if mask == '.' {
            true
        } else {
            mask == target
        }
    })
}

impl<'a> std::convert::From<CharError> for DicoError<'a> {
    fn from(e: CharError) -> Self {
        Self::Inner(e)
    }
}

impl<'a> std::fmt::Display for DicoError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DicoError::InvalidMask(mask) => write!(f, "Mask must match /[a-z\\.]+/\n  mask: {}", mask),
            DicoError::Inner(char_error) => write!(f, "Error inside the dico: {}", char_error),
            DicoError::ReadingFile => write!(f, "Error while reading file!"),
            DicoError::NoFile => write!(f, "No file named {}", DICO_NAME),
        }
    }
}
