pub type Result<T> = std::result::Result<T, failure::Error>;
use crate::util::convert_hex_to_dec;
use hex;
use merkle_sum_tree::{Leaf, MerkleSumTree, Position};
use nova_scotia::{
    circom::{
        circuit::{CircomCircuit, R1CS},
        reader::load_r1cs,
    },
    create_public_params, create_recursive_circuit, FileLocation, F, S,
};

use nova_snark::{
    traits::{circuit::TrivialTestCircuit, Group},
    CompressedSNARK, PublicParams,
};
use num::{BigInt, Num};
use pasta_curves::{Fp, Fq};
use serde_json::json;
use std::{
    collections::HashMap,
    env::current_dir,
    path::{Path, PathBuf},
    time::Instant,
};

use ff::PrimeField;
type G1 = pasta_curves::pallas::Point;
type G2 = pasta_curves::vesta::Point;

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
    root_sum: Fq,
    root_hash: Fq,
    valid_sum_hash: Fq,
    all_small_range: Fq,
}

#[derive(Debug, Clone)]
pub struct InclusionOutput {
    valid_sum_hash: Fq,
    root_sum: Fq,
    root_hash: Fq,
    user_balance: Fq,
    user_hash: Fq,
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
pub struct InclusionInput {
    user_hash: String,
    user_balance: i32,
    root_hash: String,
    root_sum: i32,
    neighbors_sum: Vec<i32>,
    neighbor_hash: Vec<String>,
    neighors_binary: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProofOfLiabilities {
    input: Vec<LiabilitiesInput>,
    output: LiabilitiesOutput,
    proof: (Vec<Fq>, Vec<Fp>),
}

#[derive(Debug, Clone)]
pub struct ProofOfInclusion {
    input: Vec<InclusionInput>,
    output: InclusionOutput,
    proof: (Vec<Fq>, Vec<Fp>),
}

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
}

impl ProofOfLiabilities {
    pub fn new(
        liabilities_inputs: Vec<LiabilitiesInput>,
        circuit_setup: &CircuitSetup,
    ) -> Result<(ProofOfLiabilities)> {
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
        let start = Instant::now();
        let res = recursive_snark.verify(
            circuit_setup.get_pp(),
            iteration_count,
            &start_public_input,
            &z0_secondary,
        );
        assert!(res.is_ok());
        let liabilities_output = LiabilitiesOutput::new(res.as_ref().unwrap());
        assert!(res.as_ref().unwrap().0[0] == F::<G1>::from(1));
        assert!(res.as_ref().unwrap().0[1] == F::<G1>::from(1));
        assert!(
            res.as_ref().unwrap().0[2]
                == F::<G1>::from_str_vartime(
                    convert_hex_to_dec(final_root_hash.to_string()).as_str()
                )
                .unwrap(),
        );
        assert!(res.as_ref().unwrap().0[3] == F::<G1>::from(final_root_sum as u64));
        println!("RecursiveSNARK::verify took {:?}", start.elapsed());
        let liabilities_proof = ProofOfLiabilities {
            input: liabilities_inputs,
            output: liabilities_output.unwrap(),
            proof: res?,
        };
        Ok((liabilities_proof))
    }
}

impl ProofOfInclusion {
    pub fn new(
        inclusion_inputs: Vec<InclusionInput>,
        circuit_setup: &CircuitSetup,
    ) -> Result<(ProofOfInclusion)> {
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
        let start = Instant::now();
        let res = recursive_snark.verify(
            circuit_setup.get_pp(),
            iteration_count,
            &start_public_input,
            &z0_secondary,
        );
        assert!(res.is_ok());
        let inclusion_output = InclusionOutput::new(res.as_ref().unwrap());
        assert!(res.as_ref().unwrap().0[0] == F::<G1>::from(1));
        println!("RecursiveSNARK::verify took {:?}", start.elapsed());
        let inclusion_proof = ProofOfInclusion {
            input: inclusion_inputs,
            output: inclusion_output.unwrap(),
            proof: res?,
        };
        Ok((inclusion_proof))
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
