//! Merkle tree bridge for VEX-HALT audit proofs
//!
//! Uses vex-core for the underlying cryptographic implementation.

use serde::{Deserialize, Serialize};
pub use vex_core::merkle::{MerkleTree as VexMerkleTree, Hash as VexHash};

/// Merkle tree wrapper for benchmark compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    pub root_hash_str: String,
    pub leaves: Vec<String>,
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            root_hash_str: hash_data("empty"),
            leaves: Vec::new(),
        }
    }

    /// Build a Merkle tree from a list of data items
    pub fn from_items(items: &[&str]) -> Self {
        if items.is_empty() {
            return Self::new();
        }

        let leaves_data: Vec<String> = items.iter().map(|s| s.to_string()).collect();
        let vex_leaves: Vec<(String, VexHash)> = items.iter().enumerate()
            .map(|(i, s)| (format!("audit_{}", i), VexHash::digest(s.as_bytes())))
            .collect();
        
        let tree = VexMerkleTree::from_leaves(vex_leaves);
        let root_hash_str = tree.root_hash()
            .map(|h| h.to_hex())
            .unwrap_or_else(|| hash_data("empty"));

        Self {
            root_hash_str,
            leaves: leaves_data,
        }
    }

    /// Get the root hash
    pub fn root_hash(&self) -> String {
        self.root_hash_str.clone()
    }


}

impl Default for MerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash data using SHA-256 (via VEX Hash)
pub fn hash_data(data: &str) -> String {
    VexHash::digest(data.as_bytes()).to_hex()
}

/// Hash multiple items together
pub fn hash_items(items: &[&str]) -> String {
    let combined = items.join("|");
    hash_data(&combined)
}

/// Create a context hash for a test execution
pub fn create_context_hash(
    test_id: &str,
    prompt: &str,
    response: &str,
    timestamp: &str,
) -> String {
    hash_items(&[test_id, prompt, response, timestamp])
}
