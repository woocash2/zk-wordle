use ff::{Field, PrimeField};
use poseidon_rs::{Fr, Poseidon};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    EmptyWordList,
    FrCreateFail,
    MerkleHashFail,
    NoSuchWord,
    WordHashFail,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeType {
    Left,
    Right,
}

pub struct MerklePathEntry {
    pub left: Fr,
    pub right: Fr,
    pub on_path: NodeType, // which one is on the path to the root, left or right
}

pub struct MerkleTree {
    m: usize,
    word_to_idx: HashMap<String, usize>,
    hashes: Vec<Fr>,
}

impl MerkleTree {
    #[allow(clippy::needless_range_loop)]
    pub fn new(words: Vec<String>) -> Result<Self, Error> {
        // get the last word which will be used to fill the bottom level of the tree
        // so that its size is a power of 2
        let last_word = words.last().ok_or(Error::EmptyWordList)?;

        // get the size of the hashes array
        let n = words.len();
        let mut m = 1;
        while m < n {
            m *= 2;
        }

        // initialize the hashes array with dummy 0 values
        let mut hashes = vec![Fr::zero(); 2 * m];

        // get the bottom level hashes of words
        let p = poseidon_rs::Poseidon::new();
        for i in m..2 * m {
            let word = words.get(i - m).unwrap_or(last_word);
            hashes[i] = word_hash(word, &p)?;
        }

        // fill the upper levels
        for i in (1..m - 1).rev() {
            hashes[i] = merkle_hash(hashes[2 * i], hashes[2 * i + 1], &p)?;
        }

        // create the word to idx mapping
        let word_to_idx: HashMap<_, _> =
            words.into_iter().enumerate().map(|(i, w)| (w, i)).collect();

        Ok(MerkleTree {
            m,
            word_to_idx,
            hashes,
        })
    }

    pub fn get_path(&self, word: &str) -> Result<Vec<MerklePathEntry>, Error> {
        // get the index of the word in the tree
        let idx = self.word_to_idx.get(word).ok_or(Error::NoSuchWord)?;
        let mut tree_idx = self.m + idx;

        // complete the path entries
        let mut path = Vec::new();
        while tree_idx > 1 {
            if tree_idx % 2 == 0 {
                // the selected node is of type left
                path.push(MerklePathEntry {
                    left: self.hashes[tree_idx],
                    right: self.hashes[tree_idx + 1],
                    on_path: NodeType::Left,
                })
            } else {
                // the selected node is of type right
                path.push(MerklePathEntry {
                    left: self.hashes[tree_idx - 1],
                    right: self.hashes[tree_idx],
                    on_path: NodeType::Right,
                })
            }
            tree_idx /= 2;
        }
        Ok(path)
    }

    pub fn root_hash(&self) -> Fr {
        self.hashes[1]
    }
}

fn merkle_hash(a: Fr, b: Fr, p: &Poseidon) -> Result<Fr, Error> {
    let input = vec![a, b];
    p.hash(input).map_err(|_| Error::MerkleHashFail)
}

fn word_hash(word: &str, p: &Poseidon) -> Result<Fr, Error> {
    let mut letter_ids = Vec::with_capacity(word.len());
    for b in word.bytes() {
        let num = b - 65; // assumes alpha string in UPPERCASE
        let fr = Fr::from_str(&num.to_string()).ok_or(Error::FrCreateFail)?;
        letter_ids.push(fr);
    }
    p.hash(letter_ids).map_err(|_| Error::WordHashFail)
}

#[cfg(test)]
mod test {
    use ff::PrimeField;
    use poseidon_rs::{Fr, Poseidon};

    use crate::{word_hash, Error, MerkleTree, NodeType};

    #[test]
    fn word_hash_correct() {
        let word = "BCZAD";
        let word_repr = [1, 2, 25, 0, 3]
            .into_iter()
            .map(|x| Fr::from_str(&x.to_string()).unwrap())
            .collect();
        let p = Poseidon::new();

        let expected_hash = p.hash(word_repr).unwrap();
        let actual_hash = word_hash(word, &p).unwrap();

        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn merkle_tree_correct() {
        let words = ["AAAAA", "BBBBB", "CCCCC", "DDDDD", "EEEEE", "FFFFF"]
            .into_iter()
            .map(|w| w.into())
            .collect();
        let tree = MerkleTree::new(words).expect("tree creation should succeed");

        let path = tree.get_path("CCCCC").expect("path should exist");

        assert_eq!(path.len(), 3);

        let p = Poseidon::new();
        let c_repr = [2; 5]
            .into_iter()
            .map(|x| Fr::from_str(&x.to_string()).unwrap())
            .collect();
        let d_repr = [3; 5]
            .into_iter()
            .map(|x| Fr::from_str(&x.to_string()).unwrap())
            .collect();

        let c_hash = p.hash(c_repr).unwrap();
        let d_hash = p.hash(d_repr).unwrap();

        assert_eq!(path[0].left, c_hash);
        assert_eq!(path[0].right, d_hash);
        assert_eq!(path[0].on_path, NodeType::Left);

        let cd_hash = p.hash(vec![c_hash, d_hash]).unwrap();
        assert_eq!(path[1].right, cd_hash);
        assert_eq!(path[1].on_path, NodeType::Right);

        let nxt_hash = p.hash(vec![path[1].left, path[1].right]).unwrap();
        assert_eq!(path[2].left, nxt_hash);
        assert_eq!(path[2].on_path, NodeType::Left);

        let top_hash = p.hash(vec![path[2].left, path[2].right]).unwrap();
        assert_eq!(top_hash, tree.root_hash());
    }

    #[test]
    fn no_word_in_tree() {
        let words = ["AAAAA", "BBBBB", "CCCCC", "DDDDD", "EEEEE", "FFFFF"]
            .into_iter()
            .map(|w| w.into())
            .collect();
        let tree = MerkleTree::new(words).expect("tree creation should succeed");

        let res = tree.get_path("ABCDE");

        assert!(res.is_err());
        if let Err(e) = res {
            assert_eq!(e, Error::NoSuchWord);
        }
    }
}
