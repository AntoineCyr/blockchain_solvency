// Integration tests for ZK proof generation and verification

use blockchain_rust::proofs::inclusion::{InclusionInput, ProofOfInclusion};
use blockchain_rust::proofs::liabilities::{LiabilitiesInput, MerkleSumTreeChange, ProofOfLiabilities};
use blockchain_rust::proofs::setup::CircuitSetup;
use merkle_sum_tree::{Leaf, MerkleSumTree};
use std::sync::Arc;

fn setup_test_tree() -> (Arc<MerkleSumTree>, Arc<MerkleSumTree>) {
    let leaf_alice_old = Leaf::new("alice".to_string(), 50);
    let leaf_alice_new = Leaf::new("alice".to_string(), 100);
    let leaf_empty = Leaf::new("0".to_string(), 0);

    let old_leafs = vec![
        leaf_alice_old,
        leaf_empty.clone(),
        leaf_empty.clone(),
        leaf_empty.clone(),
    ];
    let new_leafs = vec![
        leaf_alice_new,
        leaf_empty.clone(),
        leaf_empty.clone(),
        leaf_empty,
    ];

    let old_tree = Arc::new(MerkleSumTree::new(old_leafs).unwrap());
    let new_tree = Arc::new(MerkleSumTree::new(new_leafs).unwrap());

    (old_tree, new_tree)
}

#[test]
fn test_liabilities_proof() {
    let (old_tree, new_tree) = setup_test_tree();

    let change = MerkleSumTreeChange::new(0, old_tree, new_tree);
    let liabilities_input = LiabilitiesInput::new(vec![change]).unwrap();

    let circuit_setup = CircuitSetup::new("liabilities_changes_folding");
    let (proof, pp) = ProofOfLiabilities::new(vec![liabilities_input], &circuit_setup).unwrap();

    let result = proof.verify(pp);
    assert!(result.is_ok(), "Liabilities proof verification failed: {:?}", result.err());
    println!("Liabilities proof verified successfully");
}

#[test]
fn test_inclusion_proof() {
    let (_, tree) = setup_test_tree();

    let inclusion_input = InclusionInput::new(&tree, 0).unwrap();

    let circuit_setup = CircuitSetup::new("inclusion");
    let (proof, pp) = ProofOfInclusion::new(vec![inclusion_input], &circuit_setup).unwrap();

    let result = proof.verify(pp);
    assert!(result.is_ok(), "Inclusion proof verification failed: {:?}", result.err());
    println!("Inclusion proof verified successfully");
}
