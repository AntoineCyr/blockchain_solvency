//inputs, merkle root, 

pragma circom 2.0.0;
include "./merkle.circom";
include "./utils.circom";
include "../node_modules/circomlib/circuits/comparators.circom";

template liabilities(levels,changes) {
    // Constants for balance validation
    var DEFAULT_MAX_BALANCE_BITS = 100;  // Maximum bits for balance (supports up to 2^100)
    // Validate template parameters
    assert(levels > 0 && levels <= 32); 
    assert(changes > 0 && changes <= 1000);

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
    component balanceCheck[changes];

    // Iterate through each change
    for (var i = 0; i<changes; i++){
        //define first nodes values
        sumNodes[0][i][0] <== oldValues[i];
        hashNodes[0][i][0] <== oldUserHash[i];
        sumNodes[1][i][0] <== newValues[i];
        hashNodes[1][i][0] <== newUserHash[i];

        // Calculate currentSum
        currentSum = currentSum + newValues[i] - oldValues[i];
        
        // Perform non-negative balance validation (allows 0 balances)
        balanceCheck[i] = NonNegativeBalanceCheck(DEFAULT_MAX_BALANCE_BITS);
        balanceCheck[i].balance <== newValues[i];
        balanceCheck[i].out === 1;
    }

    // Assert newSum equals currentSum
    newSum === currentSum;

    // Part 2: Check validity of old and new paths
    // Ensure that old root + change = temp root    

    component switcherHash[2][changes][levels];
    component switcherSum[2][changes][levels];
    component merklesum[2][changes][levels];

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
    // Assert value is in old temp hash
    hashNodes[0][j][levels] === tempHash[j];

    // Assert new temp hash is valid
    hashNodes[1][j][levels] === tempHash[j+1];

    // Assert old sum is in tempSum
    sumNodes[0][j][levels] === tempSum[j];

    // Assert new sum is valid
    sumNodes[1][j][levels] === tempSum[j+1];
    }

    // All assertions passed, propagate input validity
    step_out[0] <== step_in[0];
    step_out[1] <== step_in[1];
    step_out[2] <== hashNodes[1][changes-1][levels];
    step_out[3] <== sumNodes[1][changes-1][levels];

}

component main {public [step_in]}= liabilities(2,1);