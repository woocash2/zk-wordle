use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

use merkle::{MerklePathEntry, MerkleTree};
use num_bigint::BigUint;
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
    // merkle root hash
    pub root_hash: BigUint,
}

const SOLUTION_WORDS_PATH: &str = "../words/possible_solutions.txt";
const OTHER_WORDS_PATH: &str = "../words/possible_solutions.txt";

pub struct WordBank {
    #[allow(dead_code)]
    tree: MerkleTree,
    words: Vec<String>,
}

impl WordBank {
    pub fn new() -> Result<Self, Error> {
        let solution_words = read_file(SOLUTION_WORDS_PATH).map_err(Error::IoFail)?;
        let other_words = read_file(OTHER_WORDS_PATH).map_err(Error::IoFail)?;
        let mut all_words = solution_words;
        all_words.extend(other_words);

        if all_words.iter().any(|w| !word_is_ok(w)) {
            return Err(Error::BadWord);
        }

        let tree = MerkleTree::new(&all_words).map_err(Error::MerkleCreateFail)?;

        Ok(WordBank {
            tree,
            words: all_words,
        })
    }

    pub fn pick_word(&self) -> PickWordResult {
        let idx = thread_rng().gen_range(0..self.words.len());
        PickWordResult {
            word: self.words[idx].clone(),
            path: self
                .tree
                .get_path(idx)
                .expect("idx should exist in merkle tree"),
            root_hash: self.tree.root_hash(),
        }
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

fn word_is_ok(word: &str) -> bool {
    word.len() == 5
        && word
            .chars()
            .all(|c| c.is_ascii_alphabetic() && c.is_lowercase())
}
