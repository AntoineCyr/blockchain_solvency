pragma circom 2.0.0;
include "../node_modules/circomlib/circuits/mimcsponge.circom";

template MerkleSum() {
    signal input L;
    signal input R;
    signal input sumL;
    signal input sumR;
    signal output root;
    signal output sum;

    component hasher = MiMCSponge(4, 220, 1);
    hasher.ins[0] <== L;
    hasher.ins[1] <== sumL;
    hasher.ins[2] <== R;
    hasher.ins[3] <== sumR;
    hasher.k <== 0;
    root <== hasher.outs[0];
    sum <== sumL + sumR;
}

