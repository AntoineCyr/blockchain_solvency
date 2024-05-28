pub type Result<T> = std::result::Result<T, failure::Error>;
use merkle_sum_tree::{Leaf, MerkleSumTree, Position};
/*const input = {
    step_in: [
      "0",
      "0",
      "11346658973375961332326525800941704040239142415932845440524726524725202286597",
      "46",
    ],
    oldEmailHash: ["10566265"],
    oldValues: [11],
    newEmailHash: ["19573022"],
    newValues: [15],
    tempHash: [
      "11346658973375961332326525800941704040239142415932845440524726524725202286597",
      "13409887132926978068627403428641016087864887179975784858831377354067398835782",
    ],
    tempSum: ["46", "50"],
    neighborsSum: [["10", "25"]],
    neighborsHash: [
      [
        "11672136",
        "4811434667398150016357199712138626920529027819147804819192874884729019971979",
      ],
    ],
    neighborsBinary: [["1", "0"]],
  };
*/
#[derive(Debug, Clone)]
pub struct MerkleSumTreeChange {
    index: usize,
    old_leaf: Leaf,
    new_leaf: Leaf,
    old_merkle_tree: MerkleSumTree,
    new_merkle_tree: MerkleSumTree,
}

#[derive(Debug, Clone)]
pub struct LiabilitiesProof {
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

impl LiabilitiesProof {
    pub fn new(changes: Vec<MerkleSumTreeChange>) -> Result<LiabilitiesProof> {
        let mut old_user_hash = vec![];
        let mut old_values = vec![];
        let mut new_user_hash = vec![];
        let mut new_values = vec![];
        let mut temp_hash = vec![];
        let mut temp_sum = vec![];
        let mut neighbors_sum = vec![];
        let mut neighbor_hash = vec![];
        let mut neighors_binary = vec![];

        temp_hash.push(changes[0].old_merkle_tree.get_root_hash().unwrap());
        temp_sum.push(changes[0].old_merkle_tree.get_root_sum().unwrap());
        for change in changes {
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
            old_user_hash.push(change.old_leaf.get_node().get_hash());
            old_values.push(change.old_leaf.get_node().get_value());
            new_user_hash.push(change.new_leaf.get_node().get_hash());
            new_values.push(change.new_leaf.get_node().get_value());
            temp_hash.push(change.new_merkle_tree.get_root_hash().unwrap());
            temp_sum.push(change.new_merkle_tree.get_root_sum().unwrap());
            let neighbors_sum_change = vec![];
            let neighbor_hash_change = vec![];
            let neighors_binary_change = vec![];
            for neighbor in old_merkle_path {
                neighbors_sum_change.push(neighbor.get_node().get_value());
                neighbor_hash_change.push(neighbor.get_node().get_hash());
                match neighbor.get_position() {
                    Position::Left => neighors_binary_change.push("0".to_string()),
                    Position::Right => neighors_binary_change.push("1".to_string()),
                }
            }
            neighbors_sum.push(neighbors_sum_change);
            neighbor_hash.push(neighbor_hash_change);
            neighors_binary.push(neighors_binary_change);
        }

        let liabilities_proof = LiabilitiesProof {
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
