pub type Result<T> = std::result::Result<T, failure::Error>;
use hex;
use merkle_sum_tree::{Leaf, MerkleSumTree, Position};
use nova_scotia::{
    circom::reader::load_r1cs, create_public_params, create_recursive_circuit, FileLocation, F, S,
};
use nova_snark::{CompressedSNARK, PublicParams};
use num::{BigInt, Num};
use serde_json::json;
use std::{collections::HashMap, env::current_dir, time::Instant};

use ff::PrimeField;

//Improvements: Compile multiple circuits for liabilities (size 8-16-...)
//Server create proof -> Client verify proof
//separate pp creation

#[derive(Debug, Clone)]
pub struct MerkleSumTreeChange {
    index: usize,
    old_merkle_tree: MerkleSumTree,
    new_merkle_tree: MerkleSumTree,
}

#[derive(Debug, Clone)]
pub struct LiabilitiesOutput {
    root_sum: i32,
    root_hash: String,
    not_negative: bool,
    all_small_ranger: bool,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct LiabilitiesProof {
    input: LiabilitiesInput,
    output: LiabilitiesOutput,
    proof: String,
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

        let liabilities_proof = LiabilitiesInput {
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
        Ok(liabilities_proof)
    }
}

impl LiabilitiesProof {
    pub fn new(liabilities_inputs: Vec<LiabilitiesInput>) -> Result<()> {
        type G1 = pasta_curves::pallas::Point;
        type G2 = pasta_curves::vesta::Point;
        let iteration_count = liabilities_inputs.len();
        let initial_root_hash = liabilities_inputs[0].temp_hash[0].clone();
        let initial_root_sum = liabilities_inputs[0].temp_sum[0];

        let root = current_dir().unwrap();
        let circuit_file = root.join("circuits/liabilities_changes_folding.r1cs");
        let witness_generator_file =
            root.join("circuits/liabilities_changes_folding_js/liabilities_changes_folding.wasm");

        println!("{:?}", circuit_file);
        let start_proof = Instant::now();
        let r1cs = load_r1cs::<G1, G2>(&FileLocation::PathBuf(circuit_file));

        let pp: PublicParams<G1, G2, _, _> = create_public_params(r1cs.clone());

        let mut private_inputs = Vec::new();
        for liabilities_input in liabilities_inputs {
            println!("HERE------------------------------------");
            println!("{:?}", liabilities_input.temp_sum);
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
            FileLocation::PathBuf(witness_generator_file),
            r1cs,
            private_inputs,
            start_public_input.to_vec(),
            &pp,
        )
        .unwrap();
        println!("RecursiveSNARK::proof took {:?}", start_proof.elapsed());
        let z0_secondary = [F::<G2>::from(0)];
        println!("Verifying a RecursiveSNARK...");
        let start = Instant::now();
        let res = recursive_snark.verify(&pp, iteration_count, &start_public_input, &z0_secondary);

        println!(
            "RecursiveSNARK::verify: {:?}, took {:?}",
            res,
            start.elapsed()
        );
        assert!(res.is_ok());
        Ok(())
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
fn convert_hex_to_dec(hex_str: String) -> String {
    BigInt::from_str_radix(hex_str.as_str().strip_prefix("0x").unwrap(), 16)
        .unwrap()
        .to_string()
}
