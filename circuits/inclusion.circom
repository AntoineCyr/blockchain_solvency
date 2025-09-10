pragma circom 2.0.0;
include "./merkle.circom";
include "./utils.circom";
include "../node_modules/circomlib/circuits/comparators.circom";

// Define a template for inclusion proof circuit
template inclusion(levels) {
    // Validate template parameters
    assert(levels > 0 && levels <= 32);
    
    // Define inputs
    signal input neighborsSum[levels];
    signal input neighborsHash[levels];
    signal input neighborsBinary[levels];
    signal input step_in[4];
    signal input sum;
    signal input rootHash;
    signal input userBalance;
    signal input userHash;

    // Define outputs
    component merklesumi = MerkleSum();
    merklesumi.L <== neighborsHash[0];
    merklesumi.R <== userHash;
    merklesumi.sumL <== neighborsSum[0];
    merklesumi.sumR <== userBalance;

    signal output step_out[4];
    step_out[0] <== sum;
    step_out[1] <== rootHash; 
    step_out[2] <== userBalance;
    step_out[3] <== userHash;

    // Initialize sum and hash nodes
    signal sumNodes[levels+1];
    signal hashNodes[levels+1];
    sumNodes[0] <== userBalance;
    hashNodes[0] <== userHash;

    // Define switchers and Merkle sum components
    component switcherHash[levels];
    component switcherSum[levels];
    component merklesum[levels];

    // Iterate through each level
    for (var i = 0; i < levels; i++) {
        // Connect switchers and Merkle sum components
        switcherHash[i] = Switcher();
        switcherHash[i].sel <== neighborsBinary[i];
        switcherHash[i].L <== hashNodes[i];
        switcherHash[i].R <== neighborsHash[i];

        switcherSum[i] = Switcher();
        switcherSum[i].sel <== neighborsBinary[i];
        switcherSum[i].L <== sumNodes[i];
        switcherSum[i].R <== neighborsSum[i];

        merklesum[i] = MerkleSum();
        merklesum[i].L <== switcherHash[i].outL;
        merklesum[i].R <== switcherHash[i].outR;
        merklesum[i].sumL <== switcherSum[i].outL;
        merklesum[i].sumR <== switcherSum[i].outR;

        // Update sum and hash nodes
        sumNodes[i+1] <== merklesum[i].sum;
        hashNodes[i+1] <== merklesum[i].root;
    }

    // Assert root hash is valid
    hashNodes[levels] === rootHash;

    // Assert sum is valid
    sumNodes[levels] === sum;
}

// Define main component
component main {public [step_in]}= inclusion(2);
