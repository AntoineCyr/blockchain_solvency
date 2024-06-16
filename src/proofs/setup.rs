use nova_scotia::{
    circom::{
        circuit::{CircomCircuit, R1CS},
        reader::load_r1cs,
    },
    create_public_params, FileLocation,
};
use nova_snark::{
    traits::{circuit::TrivialTestCircuit, Group},
    PublicParams,
};
use pasta_curves::Fq;
use std::{env::current_dir, path::PathBuf};

type G1 = pasta_curves::pallas::Point;
type G2 = pasta_curves::vesta::Point;

//#[derive(Debug, Clone)]
pub struct CircuitSetup {
    pp: PublicParams<
        G1,
        G2,
        CircomCircuit<<G1 as Group>::Scalar>,
        TrivialTestCircuit<<G2 as Group>::Scalar>,
    >,
    witness_generator_file: PathBuf,
    r1cs: R1CS<Fq>,
}

impl CircuitSetup {
    pub fn new(circuit_name: &str) -> CircuitSetup {
        let root = current_dir().unwrap();
        let circuit_file = root.join("circuits/".to_string() + circuit_name + ".r1cs");
        let witness_generator_file =
            root.join("circuits/".to_string() + circuit_name + "_js/" + circuit_name + ".wasm");

        let r1cs = load_r1cs::<G1, G2>(&FileLocation::PathBuf(circuit_file));

        let pp: PublicParams<G1, G2, _, _> = create_public_params(r1cs.clone());
        CircuitSetup {
            pp,
            witness_generator_file,
            r1cs,
        }
    }

    pub fn get_r1cs(&self) -> R1CS<Fq> {
        self.r1cs.clone()
    }

    pub fn get_pp(
        &self,
    ) -> &PublicParams<
        G1,
        G2,
        CircomCircuit<<G1 as Group>::Scalar>,
        TrivialTestCircuit<<G2 as Group>::Scalar>,
    > {
        &self.pp
    }

    pub fn get_witness_generator_file(&self) -> PathBuf {
        self.witness_generator_file.clone()
    }
}
