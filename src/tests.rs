#[cfg(test)]
mod tests {
    use crate::proofs::inclusion::InclusionInput;
    use crate::proofs::util::convert_hex_to_dec;
    use merkle_sum_tree::{Leaf, MerkleSumTree};

    #[test]
    fn test_liabilities_proof_direct() {
        use crate::proofs::liabilities::{LiabilitiesInput, MerkleSumTreeChange, ProofOfLiabilities};
        use crate::proofs::setup::CircuitSetup;
        use std::sync::Arc;
        use std::time::Instant;

        let circuit_setup = CircuitSetup::new("liabilities_changes_folding");
        
        // Create a simple merkle tree change
        let leaf_alice_old = Leaf::new("alice".to_string(), 50);
        let leaf_alice_new = Leaf::new("alice".to_string(), 100);
        let leaf_empty = Leaf::new("0".to_string(), 0);
        
        let old_leafs = vec![leaf_alice_old, leaf_empty.clone(), leaf_empty.clone(), leaf_empty.clone()];
        let new_leafs = vec![leaf_alice_new, leaf_empty.clone(), leaf_empty.clone(), leaf_empty];
        
        let old_tree = Arc::new(MerkleSumTree::new(old_leafs).unwrap());
        let new_tree = Arc::new(MerkleSumTree::new(new_leafs).unwrap());
        
        let change = MerkleSumTreeChange::new(0, old_tree, new_tree);
        let liabilities_input = LiabilitiesInput::new(vec![change]).unwrap();
        
        println!("Creating liabilities proof...");
        let start = Instant::now();
        let (proof, pp) = ProofOfLiabilities::new(vec![liabilities_input], &circuit_setup).unwrap();
        println!("Liabilities proof created in {:?}", start.elapsed());
        
        println!("Verifying liabilities proof...");
        let start = Instant::now();
        let verification_result = proof.verify(pp);
        println!("Liabilities verification took {:?}", start.elapsed());
        
        match verification_result {
            Ok(output) => {
                println!("Liabilities proof verified successfully!");
                println!("Output: {:#?}", output);
            }
            Err(e) => {
                println!("Liabilities verification failed: {:#?}", e);
                panic!("Test failed");
            }
        }
    }

    #[test]
    fn test_inclusion_proof_direct() {
        use nova_scotia::circom::reader::load_r1cs;
        use nova_scotia::{create_public_params, create_recursive_circuit, FileLocation, F};
        use nova_snark::PublicParams;
        use serde_json::json;
        use std::collections::HashMap;
        use std::env::current_dir;
        use std::time::Instant;

        let iteration_count = 1;
        type G1 = pasta_curves::pallas::Point;
        type G2 = pasta_curves::vesta::Point;

        let root = current_dir().unwrap();
        let circuit_file = root.join("circuits/compile/inclusion.r1cs");
        let witness_generator_file = root.join("circuits/compile/inclusion_js/inclusion.wasm");

        let r1cs = load_r1cs::<G1, G2>(&FileLocation::PathBuf(circuit_file));

        let pp: PublicParams<G1, G2, _, _> = create_public_params(r1cs.clone());

        let leaf_alice = Leaf::new("alice".to_string(), 100);
        let leaf_empty = Leaf::new("0".to_string(), 0);
        let leafs = vec![
            leaf_alice,
            leaf_empty.clone(),
            leaf_empty.clone(),
            leaf_empty,
        ];

        let tree = MerkleSumTree::new(leafs).unwrap();
        let inclusion_input = InclusionInput::new(&tree, 0).unwrap();

        let tree_proof = tree.get_proof(0).unwrap();
        let path = tree_proof.get_path();

        let mut neighbors_sum = Vec::new();
        let mut neighbors_hash = Vec::new();
        let mut neighbors_binary = Vec::new();

        for neighbor in path.iter() {
            neighbors_sum.push(neighbor.get_node().get_value());
            let hex_hash = neighbor.get_node().get_hash().to_string();
            let dec_hash = convert_hex_to_dec(hex_hash);
            neighbors_hash.push(dec_hash);

            match neighbor.get_position() {
                merkle_sum_tree::Position::Left => neighbors_binary.push("1".to_string()),
                merkle_sum_tree::Position::Right => neighbors_binary.push("0".to_string()),
            }
        }

        let root_hash_dec = convert_hex_to_dec(inclusion_input.get_root_hash().to_string());
        let user_hash_dec = convert_hex_to_dec(inclusion_input.get_user_hash().to_string());

        let mut private_inputs = Vec::new();
        for _i in 0..iteration_count {
            let mut private_input = HashMap::new();
            private_input.insert("neighborsSum".to_string(), json!(neighbors_sum));
            private_input.insert("neighborsHash".to_string(), json!(neighbors_hash));
            private_input.insert("neighborsBinary".to_string(), json!(neighbors_binary));
            private_input.insert("step_in".to_string(), json!([0, 0, 0, 0]));
            private_input.insert("sum".to_string(), json!(inclusion_input.get_root_sum()));
            private_input.insert("rootHash".to_string(), json!(root_hash_dec));
            private_input.insert(
                "userBalance".to_string(),
                json!(inclusion_input.get_user_balance()),
            );
            private_input.insert("userHash".to_string(), json!(user_hash_dec));
            private_inputs.push(private_input);
        }

        //Default values not used
        let start_public_input = [
            F::<G1>::from(0), 
            F::<G1>::from(0),
            F::<G1>::from(0),
            F::<G1>::from(0),
        ];

        println!("Creating recursive circuit...");
        let recursive_snark = create_recursive_circuit(
            FileLocation::PathBuf(witness_generator_file),
            r1cs,
            private_inputs,
            start_public_input.to_vec(),
            &pp,
        )
        .unwrap();

        println!("Proof created successfully!");

        let z0_secondary = [F::<G2>::from(0)];
        println!("Verifying RecursiveSNARK...");
        let start = Instant::now();
        let res = recursive_snark.verify(&pp, iteration_count, &start_public_input, &z0_secondary);
        println!(
            "RecursiveSNARK::verify: {:?}, took {:?}",
            res.is_ok(),
            start.elapsed()
        );

        if res.is_ok() {
            println!("Direct inclusion proof verified successfully!");
        } else {
            println!("Direct inclusion verification failed: {:?}", res.err());
        }
    }

}
