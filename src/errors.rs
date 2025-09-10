use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum BlockchainError {
    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),
    
    #[error("Proof verification failed: {reason}")]
    ProofVerification { reason: String },
    
    #[error("Invalid transaction: {reason}")]
    InvalidTransaction { reason: String },
    
    #[error("Mutex poisoned")]
    MutexPoisoned,
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    
    #[error("Proof creation failed: {0}")]
    ProofCreation(String),
    
    #[error("Circuit setup error: {0}")]
    CircuitSetup(String),
}

// Keep backward compatibility during migration
pub type Result<T> = std::result::Result<T, failure::Error>;