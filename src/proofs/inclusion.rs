pub type Result<T> = std::result::Result<T, failure::Error>;
use crate::proofs::setup::{CircuitSetup, PP};
use merkle_sum_tree::{MerkleSumTree, Position};
use nova_scotia::circom::circuit::CircomCircuit;
use nova_scotia::{create_recursive_circuit, FileLocation, F};
use nova_snark::traits::circuit::TrivialTestCircuit;
use nova_snark::RecursiveSNARK;
use pasta_curves::{Ep, Eq, Fp, Fq};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, time::Instant};

type G1 = pasta_curves::pallas::Point;
type G2 = pasta_curves::vesta::Point;

//TODO
////Verify the step_out between each fold
#[derive(Debug, Clone)]
pub struct InclusionOutput {
    valid_sum_hash: Fq,
    root_sum: Fq,
    root_hash: Fq,
    user_balance: Fq,
    user_hash: Fq,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InclusionInput {
    user_hash: String,
    user_balance: i32,
    root_hash: String,
    root_sum: i32,
    neighbors_sum: Vec<i32>,
    neighbor_hash: Vec<String>,
    neighors_binary: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProofOfInclusion {
    recursive_snark: RecursiveSNARK<Ep, Eq, CircomCircuit<Fq>, TrivialTestCircuit<Fp>>,
    iteration_count: usize,
    start_public_input: [Fq; 5],
    z0_secondary: [Fp; 1],
    inclusion_inputs: Vec<InclusionInput>,
}

impl InclusionOutput {
    pub fn new(res: &(Vec<Fq>, Vec<Fp>)) -> Result<InclusionOutput> {
        let valid_sum_hash = res.0[0];
        let root_sum = res.0[1];
        let root_hash = res.0[2];
        let user_balance = res.0[3];
        let user_hash = res.0[4];
        let liabilities_output = InclusionOutput {
            valid_sum_hash,
            root_sum,
            root_hash,
            user_balance,
            user_hash,
        };
        Ok(liabilities_output)
    }
}

impl InclusionInput {
    pub fn new(merkle_sum_tree: MerkleSumTree, index: usize) -> Result<InclusionInput> {
        let mut neighbors_sum = vec![];
        let mut neighbor_hash = vec![];
        let mut neighors_binary = vec![];
        let node = merkle_sum_tree.get_leaf(index).unwrap().get_node();
        let user_hash = node.get_hash().to_string();
        let user_balance = node.get_value();
        let root_hash = merkle_sum_tree.get_root_hash().unwrap().to_string();
        let root_sum = merkle_sum_tree.get_root_sum().unwrap();
        let merkle_path = merkle_sum_tree
            .get_proof(index)
            .unwrap()
            .unwrap()
            .get_path();

        for neighbor in merkle_path {
            neighbors_sum.push(neighbor.get_node().get_value());
            neighbor_hash.push(neighbor.get_node().get_hash().to_string());
            match neighbor.get_position() {
                Position::Left => neighors_binary.push("1".to_string()),
                Position::Right => neighors_binary.push("0".to_string()),
            }
        }

        let inclusion_input = InclusionInput {
            user_hash,
            user_balance,
            root_hash,
            root_sum,
            neighbors_sum,
            neighbor_hash,
            neighors_binary,
        };
        Ok(inclusion_input)
    }

    pub fn get_user_hash(&self) -> String {
        self.user_hash.clone()
    }

    pub fn get_user_balance(&self) -> i32 {
        self.user_balance.clone()
    }

    pub fn get_root_hash(&self) -> String {
        self.root_hash.clone()
    }

    pub fn get_root_sum(&self) -> i32 {
        self.root_sum.clone()
    }
}

impl ProofOfInclusion {
    pub fn new(
        inclusion_inputs: Vec<InclusionInput>,
        circuit_setup: &CircuitSetup,
    ) -> Result<ProofOfInclusion> {
        let iteration_count = inclusion_inputs.len();
        let start_proof = Instant::now();
        let mut private_inputs = Vec::new();
        for inclusion_input in &inclusion_inputs {
            let mut private_input = HashMap::new();
            private_input.insert(
                "neighborsSum".to_string(),
                json!(&inclusion_input.neighbors_sum),
            );
            private_input.insert(
                "neighborsHash".to_string(),
                json!(&inclusion_input.neighbor_hash),
            );
            private_input.insert(
                "neighborsBinary".to_string(),
                json!(&inclusion_input.neighors_binary),
            );
            private_input.insert("sum".to_string(), json!(&inclusion_input.root_sum));
            private_input.insert("rootHash".to_string(), json!(&inclusion_input.root_hash));
            private_input.insert(
                "userBalance".to_string(),
                json!(&inclusion_input.user_balance),
            );
            private_input.insert("userHash".to_string(), json!(&inclusion_input.user_hash));
            private_inputs.push(private_input);
        }

        let start_public_input = [
            F::<G1>::from(1),
            F::<G1>::from(0),
            F::<G1>::from(0),
            F::<G1>::from(0),
            F::<G1>::from(0),
        ];
        let recursive_snark = create_recursive_circuit(
            FileLocation::PathBuf(circuit_setup.get_witness_generator_file()),
            circuit_setup.get_r1cs(),
            private_inputs,
            start_public_input.to_vec(),
            circuit_setup.get_pp(),
        )
        .unwrap();
        println!("RecursiveSNARK::proof took {:?}", start_proof.elapsed());
        let z0_secondary = [F::<G2>::from(0)];

        let inclusion_proof = ProofOfInclusion {
            recursive_snark,
            iteration_count,
            start_public_input,
            z0_secondary,
            inclusion_inputs,
        };
        Ok(inclusion_proof)
    }

    //TODO
    //verify every intermediate step_out
    pub fn verify(&self, pp: PP) -> Result<InclusionOutput> {
        let start = Instant::now();
        let res = self.recursive_snark.verify(
            pp.get_pp(),
            self.iteration_count,
            &self.start_public_input,
            &self.z0_secondary,
        );
        assert!(res.is_ok());
        let inclusion_output = InclusionOutput::new(res.as_ref().unwrap());
        assert!(res.as_ref().unwrap().0[0] == F::<G1>::from(1));
        println!("RecursiveSNARK::verify took {:?}", start.elapsed());
        inclusion_output
    }

    pub fn get_inclusion_inputs(&self) -> Vec<InclusionInput> {
        self.inclusion_inputs.clone()
    }
}
