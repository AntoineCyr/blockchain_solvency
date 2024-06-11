//inputs, merkle root, 

pragma circom 2.0.0;
include "./merkle.circom";
include "./utils.circom";
include "../node_modules/circomlib/circuits/comparators.circom";


template liabilities(levels,changes) {
    //number of inputs need to be == 2^n

    signal input oldUserHash[changes];
    signal input oldValues[changes];
    signal input newUserHash[changes];
    signal input newValues[changes];
    signal input tempHash[changes+1];
    signal input tempSum[changes+1];
    signal newRootHash;
    signal newSum;
    signal oldSum;
    signal oldRootHash;

    signal input neighborsSum[changes][levels];
    signal input neighborsHash[changes][levels];
    signal input neighborsBinary[changes][levels];

    signal notNegative;
    signal allSmallRange;
    signal validHash;
    signal validSum;

    signal input step_in[4];
    oldRootHash <== step_in[2];
    oldSum <== step_in[3];
    signal output step_out[4];

    newRootHash <== tempHash[changes];
    newSum <== tempSum[changes];
    oldSum === tempSum[0];
    oldRootHash === tempHash[0];
    var currentSum = oldSum;

     //Part 1:
     //check valid new values
    signal sumNodes[2][changes][levels+1];
    signal hashNodes[2][changes][levels+1];
    component rangecheck[changes];
    component negativecheck[changes];
    var tempNotBig = 0;
    var tempNotNegative = 0;
    var maxBits = 100;

    // Iterate through each change
    for (var i = 0; i<changes; i++){
        //define first nodes values
        sumNodes[0][i][0] <== oldValues[i];
        hashNodes[0][i][0] <== oldUserHash[i];
        sumNodes[1][i][0] <== newValues[i];
        hashNodes[1][i][0] <== newUserHash[i];

        // Calculate currentSum
        currentSum = currentSum + newValues[i] - oldValues[i];
        
         // Perform range check and negative check
        rangecheck[i] = RangeCheck(maxBits);
        rangecheck[i].in <== newValues[i];
        tempNotBig = rangecheck[i].out + tempNotBig;

        negativecheck[i] = NegativeCheck();
        negativecheck[i].in <== newValues[i];
        tempNotNegative = negativecheck[i].out + tempNotNegative;
    }

    // Check if all new values are within range
    component rangeEqual = IsEqual();
    rangeEqual.in <== [changes,tempNotBig];
    allSmallRange <== rangeEqual.out;

    // Check if all new values are not negative
    component negativeEqual = IsEqual();
    negativeEqual.in <== [changes,tempNotNegative];
    notNegative <== negativeEqual.out;

    // Check if newSum equals currentSum
    component equalSum = IsEqual();
    equalSum.in <== [newSum,currentSum];

    // Part 2: Check validity of old and new paths
    // Ensure that old root + change = temp root    

    component switcherHash[2][changes][levels];
    component switcherSum[2][changes][levels];
    component merklesum[2][changes][levels];
    component hashEqual[2][changes];
    component sumEqual[2][changes];

    signal tempOldHashEqual[changes+1];
    signal tempOldSumEqual[changes+1];
    signal tempValidHash[changes+1];
    signal tempValidSum [changes+1];

    tempOldHashEqual[0] <== 1;
    tempOldSumEqual[0] <== 1;
    tempValidHash[0] <== 1;
    tempValidSum[0] <== 1;

    for (var j = 0; j < changes; j++){
        for  (var i = 0; i<levels; i++){
            switcherHash[0][j][i] = Switcher();
            switcherHash[0][j][i].sel <== neighborsBinary[j][i];
            switcherHash[0][j][i].L <== hashNodes[0][j][i];
            switcherHash[0][j][i].R <== neighborsHash[j][i];

            switcherSum[0][j][i] = Switcher();
            switcherSum[0][j][i].sel <== neighborsBinary[j][i];
            switcherSum[0][j][i].L <== sumNodes[0][j][i];
            switcherSum[0][j][i].R <== neighborsSum[j][i];


            merklesum[0][j][i] = MerkleSum();
            merklesum[0][j][i].L <== switcherHash[0][j][i].outL;
            merklesum[0][j][i].R <== switcherHash[0][j][i].outR;
            merklesum[0][j][i].sumL <== switcherSum[0][j][i].outL;
            merklesum[0][j][i].sumR <== switcherSum[0][j][i].outR;


            sumNodes[0][j][i+1] <== merklesum[0][j][i].sum;
            hashNodes[0][j][i+1] <== merklesum[0][j][i].root;

            switcherHash[1][j][i] = Switcher();
            switcherHash[1][j][i].sel <== neighborsBinary[j][i];
            switcherHash[1][j][i].L <== hashNodes[1][j][i];
            switcherHash[1][j][i].R <== neighborsHash[j][i];

            switcherSum[1][j][i] = Switcher();
            switcherSum[1][j][i].sel <== neighborsBinary[j][i];
            switcherSum[1][j][i].L <== sumNodes[1][j][i];
            switcherSum[1][j][i].R <== neighborsSum[j][i];


            merklesum[1][j][i] = MerkleSum();
            merklesum[1][j][i].L <== switcherHash[1][j][i].outL;
            merklesum[1][j][i].R <== switcherHash[1][j][i].outR;
            merklesum[1][j][i].sumL <== switcherSum[1][j][i].outL;
            merklesum[1][j][i].sumR <== switcherSum[1][j][i].outR;

            
            sumNodes[1][j][i+1] <== merklesum[1][j][i].sum;
            hashNodes[1][j][i+1] <== merklesum[1][j][i].root;
        }
    //value is in old temp hash
    hashEqual[0][j] = IsEqual();
    hashEqual[0][j].in <== [hashNodes[0][j][levels],tempHash[j]];
    tempOldHashEqual[j+1] <== tempOldHashEqual[j] * hashEqual[0][j].out;

    //new temp hash is valid
    hashEqual[1][j] = IsEqual();
    hashEqual[1][j].in <== [hashNodes[1][j][levels],tempHash[j+1]];
    tempValidHash[j+1] <== tempValidHash[j] * hashEqual[1][j].out;

    //old sum is in tempSum
    sumEqual[0][j] = IsEqual();
    sumEqual[0][j].in <== [sumNodes[0][j][levels],tempSum[j]];
    tempOldSumEqual[j+1] <== tempOldSumEqual[j] * sumEqual[0][j].out;

    //new sum is valid
    sumEqual[1][j] = IsEqual();
    sumEqual[1][j].in <== [sumNodes[1][j][levels],tempSum[j+1]];
    tempValidSum[j+1] <==  tempValidSum[j] * sumEqual[1][j].out;
    }

    // Check if tempHash and tempSum are valid
    validHash <== tempValidHash[changes]*tempOldHashEqual[changes];
    validSum <== tempValidSum[changes]*tempOldSumEqual[changes];

    signal validHashSum <== validHash * validSum;
    signal validNegativeRange <== notNegative * allSmallRange;
    step_out[0] <== validHashSum * step_in[0];
    step_out[1] <== validNegativeRange * step_in[1];
    step_out[2] <== hashNodes[1][changes-1][levels];
    step_out[3] <== sumNodes[1][changes-1][levels];

}

component main {public [step_in]}= liabilities(3,1);