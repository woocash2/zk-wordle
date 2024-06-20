use ff::{Field, PrimeField};
use num::{BigUint, Num};
use poseidon_rs::{Fr, Poseidon};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    EmptyWordList,
    FrCreateFail,
    MerkleHashFail,
    OutOfBounds,
    WordHashFail,
    SaltedWordHashFail,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeType {
    Left,
    Right,
}

#[derive(Clone, Debug)]
struct FrMerklePathEntry {
    pub left: Fr,
    pub right: Fr,
    pub on_path: NodeType, // which one is on the path to the root, left or right
}

#[derive(Clone, Debug)]
pub struct MerklePathEntry {
    pub left: BigUint,
    pub right: BigUint,
    pub on_path: NodeType,
}

impl From<FrMerklePathEntry> for MerklePathEntry {
    fn from(x: FrMerklePathEntry) -> Self {
        MerklePathEntry {
            left: fr_to_biguint(x.left),
            right: fr_to_biguint(x.right),
            on_path: x.on_path,
        }
    }
}

pub struct MerkleTree {
    m: usize,
    hashes: Vec<Fr>,
}

impl MerkleTree {
    #[allow(clippy::needless_range_loop)]
    pub fn new(words: &[String]) -> Result<Self, Error> {
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

        Ok(MerkleTree { m, hashes })
    }

    pub fn get_path(&self, idx: usize) -> Result<Vec<MerklePathEntry>, Error> {
        Ok(self
            .get_path_inner(idx)?
            .into_iter()
            .map(|x| x.into())
            .collect())
    }

    fn get_path_inner(&self, idx: usize) -> Result<Vec<FrMerklePathEntry>, Error> {
        if idx >= self.m {
            return Err(Error::OutOfBounds);
        }
        let mut tree_idx = self.m + idx;

        // complete the path entries
        let mut path = Vec::new();
        while tree_idx > 1 {
            if tree_idx % 2 == 0 {
                // the selected node is of type left
                path.push(FrMerklePathEntry {
                    left: self.hashes[tree_idx],
                    right: self.hashes[tree_idx + 1],
                    on_path: NodeType::Left,
                })
            } else {
                // the selected node is of type right
                path.push(FrMerklePathEntry {
                    left: self.hashes[tree_idx - 1],
                    right: self.hashes[tree_idx],
                    on_path: NodeType::Right,
                })
            }
            tree_idx /= 2;
        }
        Ok(path)
    }

    pub fn root_hash(&self) -> BigUint {
        fr_to_biguint(self.root_hash_inner())
    }

    fn root_hash_inner(&self) -> Fr {
        self.hashes[1]
    }
}

pub fn hash_word_with_salt(word: &str, salt: &BigUint) -> Result<BigUint, Error> {
    let mut input = Vec::with_capacity(6);
    for c in word.bytes() {
        let letter_id = c - 97;
        input.push(
            Fr::from_str(&letter_id.to_string()).expect("string should be a correct decimal"),
        );
    }
    input.push(Fr::from_str(&salt.to_string()).expect("string should be a correct decimal"));

    let p = Poseidon::new();
    let hash = p.hash(input).map_err(|_| Error::SaltedWordHashFail)?;
    Ok(fr_to_biguint(hash))
}

fn merkle_hash(a: Fr, b: Fr, p: &Poseidon) -> Result<Fr, Error> {
    let input = vec![a, b];
    p.hash(input).map_err(|_| Error::MerkleHashFail)
}

fn word_hash(word: &str, p: &Poseidon) -> Result<Fr, Error> {
    let mut letter_ids = Vec::with_capacity(word.len());
    for b in word.bytes() {
        let num = b - 97; // assumes alpha string in lowercase
        let fr = Fr::from_str(&num.to_string()).ok_or(Error::FrCreateFail)?;
        letter_ids.push(fr);
    }
    p.hash(letter_ids).map_err(|_| Error::WordHashFail)
}

fn fr_to_biguint(fr: Fr) -> BigUint {
    let string = fr.to_string(); // "Fr(0x<hex>)"
    let hex_string = &string[5..string.len() - 1];
    BigUint::from_str_radix(hex_string, 16).expect("hex string of Fr should be correct")
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use ff::{Field, PrimeField};
    use num::BigUint;
    use poseidon_rs::{Fr, Poseidon};

    use crate::{fr_to_biguint, word_hash, Error, MerkleTree, NodeType};

    #[test]
    fn word_hash_correct() {
        let word = "bczad";
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
        let words: Vec<_> = ["aaaaa", "bbbbb", "ccccc", "ddddd", "eeeee", "fffff"]
            .into_iter()
            .map(|w| w.into())
            .collect();
        let tree = MerkleTree::new(&words).expect("tree creation should succeed");

        let path = tree.get_path_inner(2).expect("path should exist"); // CCCCC

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
        assert_eq!(top_hash, tree.root_hash_inner());
    }

    #[test]
    fn access_out_of_bounds() {
        let words: Vec<_> = ["aaaaa", "bbbbb", "ccccc", "ddddd", "eeeee", "fffff"]
            .into_iter()
            .map(|w| w.into())
            .collect();
        let tree = MerkleTree::new(&words).expect("tree creation should succeed");

        let res = tree.get_path_inner(8); // out of bounds

        assert!(res.is_err());
        if let Err(e) = res {
            assert_eq!(e, Error::OutOfBounds);
        }
    }

    #[test]
    fn fr_to_biguint_correct() {
        let p = Poseidon::new();
        let input = vec![Fr::zero(), Fr::zero()];
        let hash = p.hash(input).unwrap();

        let actual = fr_to_biguint(hash);
        let expected = BigUint::from_str(
            "14744269619966411208579211824598458697587494354926760081771325075741142829156", // matches circom's poseidon output
        )
        .unwrap();

        assert_eq!(actual, expected);
    }
}
