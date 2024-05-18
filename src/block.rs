pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Clone)]
pub struct Block {
    transactions: Vec<Transaction>,
    prev_block_hash: i32,
    hash: i32,
    nonce: i32,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    from: String,
    to: String,
    amount: i32,
}

#[derive(Debug, Clone)]
pub struct Leaf {
    address: String,
    balance: i32,
}

impl Block {
    pub fn get_hash(&self) -> i32 {
        self.hash
    }

    /// NewBlock creates and returns Block
    pub fn new_block(transactions: Vec<Transaction>, prev_block_hash: i32) -> Block {
        let mut block = Block {
            transactions,
            prev_block_hash,
            hash: prev_block_hash + 1,
            nonce: 0,
        };
        block
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

impl Leaf {
    pub fn new(address: String, balance: i32) -> Leaf {
        let leaf = Leaf { address, balance };
        leaf
    }

    pub fn get_address(&self) -> String {
        self.address.clone()
    }

    pub fn get_balance(&self) -> i32 {
        self.balance.clone()
    }

}
