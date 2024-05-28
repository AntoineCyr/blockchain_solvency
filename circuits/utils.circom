pragma circom 2.0.0;
include "../node_modules/circomlib/circuits/comparators.circom";

template NegativeCheck(){
    signal input in;
    signal output out;

    component lessthan = LessThan(252);
    lessthan.in[0] <== -1;
    lessthan.in[1] <== in;
    out <== lessthan.out;

}

template RangeCheck(b){
    //does not handle well inputs between 2**252 and 2**254
    signal input in;
    signal output out;
    signal bits[254];
    signal out2;

    var lc = 2**b;
    
    component less_than = LessThan(252);
    less_than.in <== [in,lc];
    out <== less_than.out;
    }

template IfThenElse() {
    signal input cond;
    signal input L;
    signal input R;
    signal output out;

    out <== cond * (L - R) + R;
}

template AND() {
    signal input a;
    signal input b;
    signal output out;

    out <== a*b;
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
