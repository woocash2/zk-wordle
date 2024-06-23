use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader},
};

use merkle::{MerklePathEntry, MerkleTree};
use rand::{thread_rng, Rng};

#[derive(Debug)]
pub enum Error {
    BadWord,
    IoFail(io::Error),
    MerkleCreateFail(merkle::Error),
}

pub struct PickWordResult {
    // randomly selected word from the word bank
    pub word: String,
    // merkle siblings which contain a path to the merkle root
    pub path: Vec<MerklePathEntry>,
}

const SOLUTION_WORDS_PATH: &str = "../words/possible_solutions.txt";
const OTHER_WORDS_PATH: &str = "../words/other_valid.txt";

/// Represents the collection of the solution words, and all acceptable guess words.
/// Maintains a vector of solution words (so that picking a random one is easy) and a hash set
/// of all words (so that checking if a guess word is correct is easy).
pub struct WordBank {
    tree: MerkleTree,
    solution_words: Vec<String>, // contains solution words which correspond to merkle leaves
    all_words: HashSet<String>,  // includes all acceptable guess words
}

impl WordBank {
    /// Creates a new word bank. Reads solution words, and other acceptable guess words from files,
    /// and creates a merkle tree on top of only the solution words.
    pub fn new() -> Result<Self, Error> {
        let solution_words = read_file(SOLUTION_WORDS_PATH).map_err(Error::IoFail)?;
        let other_words = read_file(OTHER_WORDS_PATH).map_err(Error::IoFail)?;

        let mut all_words = HashSet::from_iter(other_words);
        for w in solution_words.iter() {
            all_words.insert(w.clone());
        }

        if all_words.iter().any(|w| !is_word_ok(w)) {
            return Err(Error::BadWord);
        }

        let tree = MerkleTree::new(&solution_words).map_err(Error::MerkleCreateFail)?;

        Ok(WordBank {
            tree,
            solution_words,
            all_words,
        })
    }

    /// Randomly picks a solution word, and fetches the corresponding path in the merkle tree.
    pub fn pick_word(&self) -> PickWordResult {
        let idx = thread_rng().gen_range(0..self.solution_words.len());
        PickWordResult {
            word: self.solution_words[idx].clone(),
            path: self
                .tree
                .get_path(idx)
                .expect("idx should exist in merkle tree"),
        }
    }

    /// Used to verify if a guess word is acceptable to produce a clue.
    pub fn has_word(&self, word: &str) -> bool {
        self.all_words.contains(word)
    }
}

fn read_file(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut words = Vec::new();

    for line in reader.lines() {
        let content = line?;
        words.push(content);
    }

    Ok(words)
}

// Checks word correctness syntactically.
fn is_word_ok(word: &str) -> bool {
    word.len() == 5
        && word
            .chars()
            .all(|c| c.is_ascii_alphabetic() && c.is_lowercase())
}

#[cfg(test)]
mod test {
    use crate::word_bank::is_word_ok;

    #[test]
    fn word_ok() {
        assert!(is_word_ok("abxzy"));
        assert!(is_word_ok("satuk"));
        assert!(is_word_ok("qwert"));
        assert!(is_word_ok("gamma"));
        assert!(is_word_ok("iucrp"));
    }

    #[test]
    fn word_not_ok_uppercase() {
        assert!(!is_word_ok("abXzy"));
        assert!(!is_word_ok("APQOW"));
        assert!(!is_word_ok("wreoW"));
    }

    #[test]
    fn word_not_ok_length() {
        assert!(!is_word_ok("abivbdr"));
        assert!(!is_word_ok("abi"));
        assert!(!is_word_ok(""));
    }

    #[test]
    fn word_not_ok_non_alpha() {
        assert!(!is_word_ok("ąęóćź"));
        assert!(!is_word_ok("98srh"));
        assert!(!is_word_ok("/.,g["));
    }
}
