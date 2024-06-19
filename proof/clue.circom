pragma circom 2.1.9;
include "circomlib/poseidon.circom";

template IsZero() {
    signal input in;
    signal output out;
    signal inv;
    inv <-- in!=0 ? 1/in : 0;
    out <== -in*inv +1;
    in*out === 0;
}

template IsEqual() {
    signal input x;
    signal input y;
    signal output out;
    component comp = IsZero();
    comp.in <== x - y;
    out <== comp.out;
}

template IsDifferent(){
    signal input x;
    signal input y;
    signal output out;
    component comp = IsEqual();
    comp.x <== x;
    comp.y <== y;
    out <== 1 - comp.out;
}

template IsEqual3() {
    signal input x;
    signal input y;
    signal input z;
    signal A;
    signal B;
    signal output out;
    component compXY = IsZero();
    compXY.in <== x - y;
    A <== compXY.out;
    component compYZ = IsZero();
    compYZ.in <== y - z;
    B <== compYZ.out;
    out <== A * B;
}

template Mul3(){
    signal input x;
    signal input y;
    signal input z;
    signal output out;
    signal temp;
    temp <== x * y;
    out <== temp * z;
}

template GetGreens(){
    signal input guess[5];
    signal input word[5];
    signal output greens[5];
    component isEqual[5];

    for (var i=0; i<5; i++){
        isEqual[i] = IsEqual();
        isEqual[i].x <== guess[i];
        isEqual[i].y <== word[i];
        greens[i] <== isEqual[i].out;
    }
}

template CheckCommit(){
    signal input word[5];
    signal input salt;
    signal input commit;

    component poseidon = Poseidon(6);
    
    for (var i=0; i<5; i++){
        poseidon.inputs[i] <== word[i];
    }
    poseidon.inputs[5] <== salt;

    poseidon.out === commit;
}

template CountLetter(){
    signal input word[5];
    signal input letter;
    signal output out;
    signal sums[5];

    component isEqual[5];

    for (var i=0; i<5; i++){
        isEqual[i] = IsEqual();
        isEqual[i].x <== letter;
        isEqual[i].y <== word[i];
        if (i==0){
            sums[i] <== isEqual[i].out;
        } else {
            sums[i] <== sums[i-1] + isEqual[i].out;
        }
    }

    out <== sums[4];
}

template CountAllignedLetter(){
    signal input word[5];
    signal input guess[5];
    signal input letter;
    signal output out;
    signal sums[5];

    component isEqual3[5];

    for (var i=0; i<5; i++){
        isEqual3[i] = IsEqual3();
        isEqual3[i].x <== letter;
        isEqual3[i].y <== word[i];
        isEqual3[i].z <== guess[i];

        if (i==0){
            sums[i] <== isEqual3[i].out;
        } else {
            sums[i] <== sums[i-1] + isEqual3[i].out;
        }
    }

    out <== sums[4];
}

template ContainsAndDec(){
    signal input counts[26];
    signal input letter;
    signal input skip;
    signal sum[26];
    signal output newCounts[26];
    signal output contains;

    component isEqual[26];
    component isDifferent[26];
    component mul3[26];

    // sum = sum_i (i == letter)*(counts[i] != 0)*(skip != 1)
    for (var i=0; i<26; i++){
       isEqual[i] = IsEqual();
       isDifferent[i] = IsDifferent();
       mul3[i] = Mul3();
       isEqual[i].x <== i;
       isEqual[i].y <== letter;
       isDifferent[i].x <== counts[i];
       isDifferent[i].y <== 0;
       mul3[i].x <== isEqual[i].out;
       mul3[i].y <== isDifferent[i].out;
       mul3[i].z <== 1 - skip;

       newCounts[i] <== counts[i] - mul3[i].out;
        if (i==0){
            sum[i] <== mul3[i].out;
        } else {
            sum[i] <== sum [i-1] + mul3[i].out;
        }
    }

    contains <== sum[25];
}

template Clue() {
     signal input word0;
    signal input word1;
    signal input word2;
    signal input word3;
    signal input word4;
    signal word[5];
    signal input guess0;
    signal input guess1;
    signal input guess2;
    signal input guess3;
    signal input guess4;
    signal guess[5];

    word[0] <== word0;
    word[1] <== word1;
    word[2] <== word2;
    word[3] <== word3;
    word[4] <== word4;

    guess[0] <== guess0;
    guess[1] <== guess1;
    guess[2] <== guess2;
    guess[3] <== guess3;
    guess[4] <== guess4;
    signal input commit;
    signal input salt;
    signal greens[5];
    signal counts[26];
    signal allignedCounts[26];
    signal yellowCounts[6][26];
    signal yellows[5];
    signal output clue[5];
    
    component checkCommit = CheckCommit();
    checkCommit.word <== word;
    checkCommit.salt <== salt;
    checkCommit.commit <== commit;

    component getGreens = GetGreens();
    getGreens.guess <== guess;
    getGreens.word <== word;
    greens <== getGreens.greens;

    component countLetter[26];
    for (var i=0; i<26; i++){
        countLetter[i] = CountLetter();
        countLetter[i].word <== word;
        countLetter[i].letter <== i;
        counts[i] <== countLetter[i].out;
    }

    component countAllignedLetter[26];
    for (var i=0; i<26; i++){
        countAllignedLetter[i] = CountAllignedLetter();
        countAllignedLetter[i].word <== word;
        countAllignedLetter[i].guess <== guess;
        countAllignedLetter[i].letter <== i;
        allignedCounts[i] <== countAllignedLetter[i].out;
    }

    for (var i=0; i<26; i++){
        yellowCounts[0][i] <== counts[i] - allignedCounts[i];
    }

    component contains[5];
    for (var i=0; i<5; i++){
        contains[i] = ContainsAndDec();
        contains[i].counts <== yellowCounts[i];
        contains[i].letter <== guess[i];
        contains[i].skip <== greens[i];
        yellowCounts[i+1] <== contains[i].newCounts;
        yellows[i] <== contains[i].contains;
    }
    
    for (var i=0; i<5; i++){
        clue[i] <== 2*greens[i] + yellows[i];
    }
    
}

component main {public [guess0,guess1,guess2,guess3,guess4, commit]} = Clue();