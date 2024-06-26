pub type Result<T> = std::result::Result<T, failure::Error>;
use crate::proofs::setup::{CircuitSetup, PP};
use crate::proofs::util::convert_hex_to_dec;
use ff::PrimeField;
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
//Improvements: Compile multiple circuits for liabilities (size 8-16-...)
//Server create proof -> Client verify proof
//Verify the step_out between each fold
#[derive(Debug, Clone)]
pub struct MerkleSumTreeChange {
    index: usize,
    old_merkle_tree: MerkleSumTree,
    new_merkle_tree: MerkleSumTree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiabilitiesOutput {
    root_sum: Fq,
    root_hash: Fq,
    valid_sum_hash: Fq,
    all_small_range: Fq,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiabilitiesInput {
    old_user_hash: Vec<String>,
    old_values: Vec<i32>,
    new_user_hash: Vec<String>,
    new_values: Vec<i32>,
    temp_hash: Vec<String>,
    temp_sum: Vec<i32>,
    neighbors_sum: Vec<Vec<i32>>,
    neighbor_hash: Vec<Vec<String>>,
    neighors_binary: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProofOfLiabilities {
    recursive_snark: RecursiveSNARK<Ep, Eq, CircomCircuit<Fq>, TrivialTestCircuit<Fp>>,
    iteration_count: usize,
    start_public_input: [Fq; 4],
    z0_secondary: [Fp; 1],
    final_root_hash: String,
    final_root_sum: i32,
}

impl LiabilitiesOutput {
    pub fn new(res: &(Vec<Fq>, Vec<Fp>)) -> Result<LiabilitiesOutput> {
        let valid_sum_hash = res.0[0];
        let all_small_range = res.0[1];
        let root_hash = res.0[2];
        let root_sum = res.0[3];
        let liabilities_output = LiabilitiesOutput {
            valid_sum_hash,
            all_small_range,
            root_hash,
            root_sum,
        };
        Ok(liabilities_output)
    }
}

impl LiabilitiesInput {
    pub fn new(changes: Vec<MerkleSumTreeChange>) -> Result<LiabilitiesInput> {
        let mut old_user_hash = vec![];
        let mut old_values = vec![];
        let mut new_user_hash = vec![];
        let mut new_values = vec![];
        let mut temp_hash = vec![];
        let mut temp_sum = vec![];
        let mut neighbors_sum = vec![];
        let mut neighbor_hash = vec![];
        let mut neighors_binary = vec![];

        temp_hash.push(
            changes[0]
                .old_merkle_tree
                .get_root_hash()
                .unwrap()
                .to_string(),
        );
        temp_sum.push(changes[0].old_merkle_tree.get_root_sum().unwrap());
        for change in changes {
            let old_leaf = change.old_merkle_tree.get_leaf(change.index).unwrap();
            let new_leaf = change.new_merkle_tree.get_leaf(change.index).unwrap();
            let old_merkle_path = change
                .old_merkle_tree
                .get_proof(change.index)
                .unwrap()
                .unwrap()
                .get_path();
            let new_merkle_path = change
                .old_merkle_tree
                .get_proof(change.index)
                .unwrap()
                .unwrap()
                .get_path();
            assert!(old_merkle_path == new_merkle_path);
            old_user_hash.push(old_leaf.get_node().get_hash().to_string());
            old_values.push(old_leaf.get_node().get_value());
            new_user_hash.push(new_leaf.get_node().get_hash().to_string());
            new_values.push(new_leaf.get_node().get_value());
            temp_hash.push(change.new_merkle_tree.get_root_hash().unwrap().to_string());
            temp_sum.push(change.new_merkle_tree.get_root_sum().unwrap());
            let mut neighbors_sum_change = vec![];
            let mut neighbor_hash_change = vec![];
            let mut neighors_binary_change = vec![];
            for neighbor in old_merkle_path {
                neighbors_sum_change.push(neighbor.get_node().get_value());
                neighbor_hash_change.push(neighbor.get_node().get_hash().to_string());
                match neighbor.get_position() {
                    Position::Left => neighors_binary_change.push("1".to_string()),
                    Position::Right => neighors_binary_change.push("0".to_string()),
                }
            }
            neighbors_sum.push(neighbors_sum_change);
            neighbor_hash.push(neighbor_hash_change);
            neighors_binary.push(neighors_binary_change);
        }

        let liabilities_input = LiabilitiesInput {
            old_user_hash,
            old_values,
            new_user_hash,
            new_values,
            temp_hash,
            temp_sum,
            neighbors_sum,
            neighbor_hash,
            neighors_binary,
        };
        Ok(liabilities_input)
    }
}

impl ProofOfLiabilities {
    pub fn new(
        liabilities_inputs: Vec<LiabilitiesInput>,
        circuit_setup: &CircuitSetup,
    ) -> Result<ProofOfLiabilities> {
        let iteration_count = liabilities_inputs.len();
        let initial_root_hash = liabilities_inputs[0].temp_hash[0].clone();
        let initial_root_sum = liabilities_inputs[0].temp_sum[0];
        let number_of_temp = liabilities_inputs[0].temp_sum.len() - 1;
        let final_root_hash =
            liabilities_inputs[iteration_count - 1].temp_hash[number_of_temp].clone();
        let final_root_sum = liabilities_inputs[iteration_count - 1].temp_sum[number_of_temp];

        let start_proof = Instant::now();
        let mut private_inputs = Vec::new();
        for liabilities_input in &liabilities_inputs {
            let mut private_input = HashMap::new();
            private_input.insert(
                "oldUserHash".to_string(),
                json!(&liabilities_input.old_user_hash),
            );
            private_input.insert(
                "oldValues".to_string(),
                json!(&liabilities_input.old_values),
            );
            private_input.insert(
                "newUserHash".to_string(),
                json!(&liabilities_input.new_user_hash),
            );
            private_input.insert(
                "newValues".to_string(),
                json!(&liabilities_input.new_values),
            );
            private_input.insert("tempHash".to_string(), json!(&liabilities_input.temp_hash));
            private_input.insert("tempSum".to_string(), json!(&liabilities_input.temp_sum));
            private_input.insert(
                "neighborsSum".to_string(),
                json!(&liabilities_input.neighbors_sum),
            );
            private_input.insert(
                "neighborsHash".to_string(),
                json!(&liabilities_input.neighbor_hash),
            );
            private_input.insert(
                "neighborsBinary".to_string(),
                json!(&liabilities_input.neighors_binary),
            );
            private_inputs.push(private_input);
        }

        let start_public_input = [
            F::<G1>::from(1),
            F::<G1>::from(1),
            F::<G1>::from_str_vartime(convert_hex_to_dec(initial_root_hash).as_str()).unwrap(),
            F::<G1>::from(initial_root_sum as u64),
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
        let liabilities_proof = ProofOfLiabilities {
            recursive_snark: recursive_snark,
            iteration_count: iteration_count,
            start_public_input: start_public_input,
            z0_secondary: z0_secondary,
            final_root_hash: final_root_hash,
            final_root_sum: final_root_sum,
        };
        Ok(liabilities_proof)
    }

    pub fn verify(&self, pp: PP) -> Result<LiabilitiesOutput> {
        let start = Instant::now();
        let res = self.recursive_snark.verify(
            pp.get_pp(),
            self.iteration_count,
            &self.start_public_input,
            &self.z0_secondary,
        );
        assert!(res.is_ok());
        let liabilities_output = LiabilitiesOutput::new(res.as_ref().unwrap());
        assert!(res.as_ref().unwrap().0[0] == F::<G1>::from(1));
        assert!(res.as_ref().unwrap().0[1] == F::<G1>::from(1));
        assert!(
            res.as_ref().unwrap().0[2]
                == F::<G1>::from_str_vartime(
                    convert_hex_to_dec(self.final_root_hash.to_string()).as_str()
                )
                .unwrap(),
        );
        assert!(res.as_ref().unwrap().0[3] == F::<G1>::from(self.final_root_sum as u64));
        println!("RecursiveSNARK::verify took {:?}", start.elapsed());
        liabilities_output
    }

    pub fn serialize(self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn deserialize(proof_of_liabilities: String) -> Result<ProofOfLiabilities> {
        match serde_json::from_str(&proof_of_liabilities) {
            Ok(data) => Ok(data),
            Err(error) => Result::Err(error.into()),
        }
    }
}

impl MerkleSumTreeChange {
    pub fn new(
        index: usize,
        old_merkle_tree: MerkleSumTree,
        new_merkle_tree: MerkleSumTree,
    ) -> MerkleSumTreeChange {
        MerkleSumTreeChange {
            index,
            old_merkle_tree,
            new_merkle_tree,
        }
    }
}
