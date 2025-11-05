pub type Result<T> = std::result::Result<T, failure::Error>;
use crate::proofs::setup::{CircuitSetup, PP};
use crate::proofs::util::convert_hex_to_dec;
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

use crate::blockchain::blockchain::MAX_LEVELS;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InclusionInput {
    user_hash: String,
    user_balance: i32,
    root_hash: String,
    root_sum: i32,
    neighbors_sum: Vec<i32>,
    neighbor_hash: Vec<String>,
    neighbors_binary: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProofOfInclusion {
    recursive_snark: RecursiveSNARK<Ep, Eq, CircomCircuit<Fq>, TrivialTestCircuit<Fp>>,
    iteration_count: usize,
    start_public_input: [Fq; 4],
    z0_secondary: [Fp; 1],
    inclusion_inputs: Vec<InclusionInput>,
}


impl InclusionInput {
    pub fn new(merkle_sum_tree: &MerkleSumTree, index: usize) -> Result<InclusionInput> {
        let mut neighbors_sum = Vec::with_capacity(MAX_LEVELS);
        let mut neighbor_hash = Vec::with_capacity(MAX_LEVELS);
        let mut neighbors_binary = Vec::with_capacity(MAX_LEVELS);
        let node = merkle_sum_tree.get_leaf(index).unwrap().get_node();
        let user_hash = node.get_hash().to_string();
        let user_balance = node.get_value();
        let root_hash = merkle_sum_tree.get_root_hash().unwrap().to_string();
        let root_sum = merkle_sum_tree.get_root_sum().unwrap();
        let proof = merkle_sum_tree
            .get_proof(index)
            .unwrap();
        let merkle_path = proof.get_path();

        
        for (_i, neighbor) in merkle_path.iter().enumerate() {
                neighbors_sum.push(neighbor.get_node().get_value());
                neighbor_hash.push(neighbor.get_node().get_hash().to_string());
                match neighbor.get_position() {
                    Position::Left => neighbors_binary.push("1".to_string()),
                    Position::Right => neighbors_binary.push("0".to_string()),
            }
        }
        
        let inclusion_input = InclusionInput {
            user_hash,
            user_balance,
            root_hash,
            root_sum,
            neighbors_sum,
            neighbor_hash,
            neighbors_binary,
        };
        Ok(inclusion_input)
    }

    pub fn get_user_balance(&self) -> i32 {
        self.user_balance
    }

    pub fn get_root_hash(&self) -> &str {
        &self.root_hash
    }

    pub fn get_root_sum(&self) -> i32 {
        self.root_sum
    }

    #[allow(dead_code)]
    pub fn get_user_hash(&self) -> &str {
        &self.user_hash
    }
}


impl ProofOfInclusion {

    pub fn new(
        inclusion_inputs: Vec<InclusionInput>,
        circuit_setup: &CircuitSetup,
    ) -> Result<(ProofOfInclusion, PP)> {
        use nova_scotia::create_public_params;
        let r1cs = circuit_setup.get_r1cs();
        let pp = create_public_params(r1cs.clone());
        
        let iteration_count = inclusion_inputs.len();
        let start_proof = Instant::now();
        let mut private_inputs = Vec::new();
        for (_, inclusion_input) in inclusion_inputs.iter().enumerate() {
            
            let mut private_input = HashMap::new();
            private_input.insert("neighborsSum".to_string(), json!(&inclusion_input.neighbors_sum));
            
            // Convert hex hashes to decimal strings like liabilities proof does
            let mut neighbors_hash_dec = Vec::new();
            for hex_hash in &inclusion_input.neighbor_hash {
                neighbors_hash_dec.push(convert_hex_to_dec(hex_hash.to_string()));
            }
            private_input.insert("neighborsHash".to_string(), json!(neighbors_hash_dec));
            private_input.insert("neighborsBinary".to_string(), json!(&inclusion_input.neighbors_binary));
            private_input.insert("sum".to_string(), json!(&inclusion_input.root_sum));
            private_input.insert("rootHash".to_string(), json!(convert_hex_to_dec(inclusion_input.root_hash.to_string())));
            private_input.insert("userBalance".to_string(), json!(&inclusion_input.user_balance));
            private_input.insert("userHash".to_string(), json!(convert_hex_to_dec(inclusion_input.user_hash.to_string())));
            
            private_inputs.push(private_input);
        }

        let start_public_input = [F::<G1>::from(0), F::<G1>::from(0), F::<G1>::from(0), F::<G1>::from(0)];
        
        let recursive_snark = create_recursive_circuit(
            FileLocation::PathBuf(circuit_setup.get_witness_generator_file().to_path_buf()),
            r1cs.clone(),
            private_inputs,
            start_public_input.to_vec(),
            &pp,
        ).unwrap();
        
        println!("RecursiveSNARK::proof took {:?}", start_proof.elapsed());
        let z0_secondary = [F::<G2>::from(0)];

        let inclusion_proof = ProofOfInclusion {
            recursive_snark,
            iteration_count,
            start_public_input,
            z0_secondary,
            inclusion_inputs,
        };
        
        // Create a PP wrapper for the client using the same r1cs  
        let client_pp = PP::new(r1cs);
        Ok((inclusion_proof, client_pp))
    }

    pub fn verify(&self, pp: PP) -> Result<()> {
        let start = Instant::now();
        let res = self.recursive_snark.verify(
            pp.get_pp(),
            self.iteration_count,
            &self.start_public_input,
            &self.z0_secondary,
        );
        
        if res.is_err() {
            return Err(failure::format_err!("Final inclusion proof verification failed: {:?}", res.err()));
        }
        
        println!("Inclusion folding verified successfully in {:?}", start.elapsed());
        
        Ok(())
    }

    pub fn get_inclusion_inputs(&self) -> Vec<InclusionInput> {
        self.inclusion_inputs.clone()
    }
}
