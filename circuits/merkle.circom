pragma circom 2.0.0;
include "../node_modules/circomlib/circuits/mimcsponge.circom";

template MerkleSum() {
    signal input L;
    signal input R;
    signal input sumL;
    signal input sumR;
    signal output root;
    signal output sum;

    // Constants for MiMC sponge parameters
    var MIMC_INPUTS = 4;      // Number of inputs for hash (L, sumL, R, sumR)
    var MIMC_ROUNDS = 220;    // Number of rounds for security 
    var MIMC_OUTPUTS = 1;     // Single hash output

    component hasher = MiMCSponge(MIMC_INPUTS, MIMC_ROUNDS, MIMC_OUTPUTS);
    hasher.ins[0] <== L;
    hasher.ins[1] <== sumL;
    hasher.ins[2] <== R;
    hasher.ins[3] <== sumR;
    hasher.k <== 0;
    root <== hasher.outs[0];
    sum <== sumL + sumR;
}

