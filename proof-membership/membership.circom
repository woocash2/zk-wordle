pragma circom 2.1.9;
include "../circomlib/poseidon.circom";

template CheckCommitment(){
    signal input word[5];
    signal input salt;
    signal input cm;

    component poseidon = Poseidon(6);
    
    for (var i=0; i<5; i++){
        poseidon.inputs[i] <== word[i];
    }
    poseidon.inputs[5] <== salt;

    poseidon.out === cm;
}

template Select() {
    signal input left;
    signal input right;
    signal input selector;
    signal output out;

    out <== left + (right - left) * selector;
}

template Membership(numLevels) {
    // the solution word encoded by a=0, b=1, ...
    signal input word[5];
    signal input salt;
    signal input cm;

    // hashes[i] are two hashes on merkle level i which are siblings.
    // hashes[i][0] is the left sibling, hashes[i][1] is the right sibling.
    signal input hashes[numLevels][2];

    // pathIndicators[i] == 0 means the left sibling on level i is on path
    // pathIndicators[i] == 1 means the right sibling on level i is on path
    signal input pathIndicators[numLevels];

    // root hash of the merkle tree
    signal output rootHash;

    // check that cm == cm(word, salt)
    component checkCm = CheckCommitment();
    checkCm.word <== word;
    checkCm.salt <== salt;
    checkCm.cm <== cm;

    // hashing component for each level
    // poseidon[0] is hash(word), poseidon[i + 1] is hash(hashes[i][0], hashes[i][1])
    component poseidon[numLevels + 1];

    // components which select one of hashes[i] based on pathIndicators[i]
    component select[numLevels];

    // compute poseidon of the word
    poseidon[0] = Poseidon(5);
    for (var i=0; i<5; i++){
        poseidon[0].inputs[i] <== word[i];
    }

    for (var i = 0; i < numLevels; i++) {
        // each path indicator should be 0 or 1
        pathIndicators[i] * (1 - pathIndicators[i]) === 0;

        // select the hash which lies on the path
        select[i] = Select();
        select[i].left <== hashes[i][0];
        select[i].right <== hashes[i][1];
        select[i].selector <== pathIndicators[i];

        // assert that the path hash is equal to the computed poseidon
        poseidon[i].out === select[i].out;

        // compute the next poseidon
        poseidon[i + 1] = Poseidon(2);
        poseidon[i + 1].inputs[0] <== hashes[i][0];
        poseidon[i + 1].inputs[1] <== hashes[i][1];
    }

    // set rootHash as the top-level poseidon
    rootHash <== poseidon[numLevels].out;
}

component main {public [cm]} = Membership(13); // our wordle-merkle has 13 levels