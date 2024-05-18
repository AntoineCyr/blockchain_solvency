pragma circom 2.0.0;
include "./merkle.circom";
include "./utils.circom";
include "../node_modules/circomlib/circuits/comparators.circom";

// Define a template for a Merkle tree circuit
template sumMerkleTree(levels, inputs) {
    // Ensure the number of inputs is a power of 2
    // Define input signals for balances and email hashes
    signal input balance[inputs];
    signal input emailHash[inputs];

    // Define output signals for sum and root hash
    signal output sum;
    signal output rootHash;

    // Define output signals for range checks
    signal output notNegative;
    signal output allSmallRange;

    // Define arrays for storing sum and hash nodes at each level
    signal sumNodes[levels + 1][inputs];
    signal hashNodes[levels + 1][inputs];

    // Initialize variables
    var levelSize = inputs;
    var nextLevelSize = 0;
    var maxBits = 100;
    var tempNotBig = 0;
    var tempNotNegative = 0;
    
    // Define range check and negative check components
    component rangecheck[inputs];
    component negativecheck[inputs];

    // Loop through each input
    for (var i = 0; i < inputs; i++) {
        // Assign input values to hash and sum nodes
        hashNodes[0][i] <== emailHash[i];
        sumNodes[0][i] <== balance[i];

        // Perform range check and negative check
        rangecheck[i] = RangeCheck(maxBits);
        rangecheck[i].in <== balance[i];
        tempNotBig = rangecheck[i].out + tempNotBig;

        negativecheck[i] = NegativeCheck();
        negativecheck[i].in <== balance[i];
        tempNotNegative = negativecheck[i].out + tempNotNegative;
    }

    // Check if all balances are within a small range
    component rangeEqual = IsEqual();
    rangeEqual.in <== [inputs, tempNotBig];
    allSmallRange <== rangeEqual.out;

    // Check if all balances are non-negative
    component negativeEqual = IsEqual();
    negativeEqual.in <== [inputs, tempNotNegative];
    notNegative <== negativeEqual.out;

    // Define Merkle sum components
    component merklesum[levels][inputs];
    // Loop through each level
    for (var i = 0; i < levels; i++) {
        // Loop through each pair of nodes at the current level
        for (var j = 0; j < levelSize; j = j + 2) {
            // Compute Merkle sum for each pair of nodes
            merklesum[i][j] = MerkleSum();
            
            // Assign input values to Merkle sum component
            merklesum[i][j].L <== hashNodes[i][j];
            merklesum[i][j].R <== hashNodes[i][j + 1];
            merklesum[i][j].sumL <== sumNodes[i][j];
            merklesum[i][j].sumR <== sumNodes[i][j + 1];

            // Store sum and root hash for the next level
            sumNodes[i + 1][nextLevelSize] <== merklesum[i][j].sum;
            hashNodes[i + 1][nextLevelSize] <== merklesum[i][j].root;
            nextLevelSize += 1;
        }
        // Update the size of the current level
        levelSize = nextLevelSize;
        nextLevelSize = 0;
    }
    // Assign final sum and root hash values
    sum <== sumNodes[levels][0];
    rootHash <== hashNodes[levels][0]; 
}

// Instantiate the main component with parameters 5 levels and 32 inputs
component main = sumMerkleTree(1, 2);
