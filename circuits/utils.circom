pragma circom 2.0.0;
include "../node_modules/circomlib/circuits/comparators.circom";
include "../node_modules/circomlib/circuits/bitify.circom";

// Secure range check using bit decomposition
template SafeRangeCheck(bits) {
    signal input in;
    signal output out;
    
    // Convert to bits and back to ensure no overflow
    component n2b = Num2Bits(bits);
    n2b.in <== in;
    
    component b2n = Bits2Num(bits);
    for (var i = 0; i < bits; i++) {
        b2n.in[i] <== n2b.out[i];
    }
    
    // Ensure round-trip works (no field overflow)
    b2n.out === in;
    out <== 1;
}

// Non-negative balance validation (accepts 0 and positive balances)
template NonNegativeBalanceCheck(maxBits) {
    signal input balance;
    signal output out;
    
    // Ensure balance fits in maxBits (implicitly non-negative in field)
    component rangeCheck = SafeRangeCheck(maxBits);
    rangeCheck.in <== balance;
    rangeCheck.out === 1;
    
    // No additional zero check - 0 balances are allowed
    out <== 1;
}


template Switcher() {
    signal input sel;
    signal input L;
    signal input R;
    signal output outL;
    signal output outR;

    signal aux;

    aux <== (R-L)*sel;
    outL <==  aux + L;
    outR <== -aux + R;
}
