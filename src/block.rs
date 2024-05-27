pub type Result<T> = std::result::Result<T, failure::Error>;
use merkle_sum_tree::{Leaf, MerkleSumTree};
use std::collections::HashMap;

//work on hash and nonce
#[derive(Debug, Clone)]
pub struct Block {
    transactions: Vec<Transaction>,
    prev_block_hash: i32,
    hash: i32,
    nonce: i32,
    state: HashMap<String, i32>,
    leaf_index: HashMap<String, usize>,
    merkle_sum_tree: MerkleSumTree,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    from: String,
    to: String,
    amount: i32,
}

impl Block {
    pub fn get_hash(&self) -> i32 {
        self.hash.clone()
    }

    pub fn get_state(&self) -> HashMap<String, i32> {
        self.state.clone()
    }

    pub fn get_merkle_sum_tree(&self) -> MerkleSumTree {
        self.merkle_sum_tree.clone()
    }

    pub fn new_block(transactions: Vec<Transaction>, prev_block: Option<Block>) -> Result<Block> {
        match prev_block {
            None => Ok(Block {
                transactions,
                prev_block_hash: 0,
                hash: 1,
                nonce: 0,
                state: HashMap::new(),
                leaf_index: HashMap::new(),
                merkle_sum_tree: MerkleSumTree::new(vec![]).unwrap(),
            }),
            Some(block) => Self::update_block(block, transactions),
        }
    }

    fn update_block(mut block: Block, transactions: Vec<Transaction>) -> Result<Block> {
        for transaction in transactions {
            let from: String = transaction.get_from();
            let to: String = transaction.get_to();
            let amount: i32 = transaction.get_amount();

            let number_from = match block.state.get(&from) {
                Some(&number) => number,
                _ => 0,
            };
            let number_to = match block.state.get(&to) {
                Some(&number) => number,
                _ => 0,
            };
            if from != "" {
                block.update_state(from, number_from - amount)?;
            }
            block.update_state(to, number_to + amount)?;
        }
        Ok(block)
    }

    fn update_state(&mut self, address: String, amount: i32) -> Result<()> {
        self.state.insert(address.clone(), amount.clone());
        let index = self.leaf_index.get(&address);

        let leaf = Leaf::new(address.clone(), amount);
        if index.is_some() {
            _ = self.merkle_sum_tree.set_leaf(leaf, *index.unwrap());
        } else {
            let index = self.merkle_sum_tree.push(leaf).unwrap();
            self.leaf_index.insert(address, index);
        }
        Ok(())
    }
}

impl Transaction {
    pub fn new(from: String, to: String, amount: i32) -> Transaction {
        let transaction = Transaction { from, to, amount };
        transaction
    }

    pub fn get_to(&self) -> String {
        self.to.clone()
    }

    pub fn get_from(&self) -> String {
        self.from.clone()
    }

    pub fn get_amount(&self) -> i32 {
        self.amount.clone()
    }
}
