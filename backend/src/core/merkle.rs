use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

/// Merkle Tree Node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MerkleNode {
    Leaf {
        hash: String,
        data_hash: String, // Hash of the actual data
    },
    Branch {
        hash: String,
        left: Box<MerkleNode>,
        right: Box<MerkleNode>,
    },
}

impl MerkleNode {
    pub fn hash(&self) -> &str {
        match self {
            MerkleNode::Leaf { hash, .. } => hash,
            MerkleNode::Branch { hash, .. } => hash,
        }
    }

    /// Create a leaf node from data
    pub fn leaf(data: &[u8]) -> Self {
        let data_hash = hex::encode(Sha256::digest(data));
        let hash = hex::encode(Sha256::digest(&data_hash.as_bytes()));
        MerkleNode::Leaf { hash, data_hash }
    }

    /// Create a branch node from two children
    pub fn branch(left: MerkleNode, right: MerkleNode) -> Self {
        let combined = format!("{}{}", left.hash(), right.hash());
        let hash = hex::encode(Sha256::digest(combined.as_bytes()));
        MerkleNode::Branch {
            hash,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

/// Merkle Proof for verifying inclusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub leaf_hash: String,
    pub proof_hashes: Vec<ProofNode>,
    pub root_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofNode {
    pub hash: String,
    pub position: Position, // Left or Right sibling
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Position {
    Left,
    Right,
}

/// Merkle Tree Builder and Verifier
pub struct MerkleTree {
    pub root: Option<MerkleNode>,
    pub leaves: Vec<MerkleNode>,
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            root: None,
            leaves: Vec::new(),
        }
    }

    /// Add a leaf to the tree
    pub fn add_leaf(&mut self, data: &[u8]) {
        self.leaves.push(MerkleNode::leaf(data));
        self.rebuild_root();
    }

    /// Rebuild the root from all leaves
    fn rebuild_root(&mut self) {
        if self.leaves.is_empty() {
            self.root = None;
            return;
        }

        let mut nodes = self.leaves.clone();
        
        while nodes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in nodes.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(MerkleNode::branch(chunk[0].clone(), chunk[1].clone()));
                } else {
                    // Odd number of nodes - promote the last one
                    next_level.push(chunk[0].clone());
                }
            }
            
            nodes = next_level;
        }

        self.root = nodes.into_iter().next();
    }

    /// Get the root hash
    pub fn root_hash(&self) -> Option<String> {
        self.root.as_ref().map(|n| n.hash().to_string())
    }

    /// Generate a proof for a specific leaf index
    pub fn generate_proof(&self, leaf_index: usize) -> Option<MerkleProof> {
        if leaf_index >= self.leaves.len() || self.root.is_none() {
            return None;
        }

        let leaf = &self.leaves[leaf_index];
        let mut proof_hashes = Vec::new();
        let mut current_index = leaf_index;
        let mut nodes = self.leaves.clone();

        while nodes.len() > 1 {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < nodes.len() {
                let position = if current_index % 2 == 0 {
                    Position::Right
                } else {
                    Position::Left
                };

                proof_hashes.push(ProofNode {
                    hash: nodes[sibling_index].hash().to_string(),
                    position,
                });
            }

            // Move to next level
            let mut next_level = Vec::new();
            for chunk in nodes.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(MerkleNode::branch(chunk[0].clone(), chunk[1].clone()));
                } else {
                    next_level.push(chunk[0].clone());
                }
            }

            nodes = next_level;
            current_index /= 2;
        }

        Some(MerkleProof {
            leaf_hash: leaf.hash().to_string(),
            proof_hashes,
            root_hash: self.root_hash()?,
        })
    }

    /// Verify a Merkle proof
    pub fn verify_proof(proof: &MerkleProof) -> bool {
        let mut current_hash = proof.leaf_hash.clone();

        for proof_node in &proof.proof_hashes {
            let combined = match proof_node.position {
                Position::Left => format!("{}{}", proof_node.hash, current_hash),
                Position::Right => format!("{}{}", current_hash, proof_node.hash),
            };
            current_hash = hex::encode(Sha256::digest(combined.as_bytes()));
        }

        current_hash == proof.root_hash
    }
}

/// Audit Log Entry with Merkle integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub event_type: String,
    pub payload_hash: String,
    pub timestamp: i64,
    pub merkle_root: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree_basic() {
        let mut tree = MerkleTree::new();
        tree.add_leaf(b"transaction1");
        tree.add_leaf(b"transaction2");
        
        assert!(tree.root.is_some());
        assert_ne!(tree.root_hash(), None);
    }

    #[test]
    fn test_merkle_proof_verification() {
        let mut tree = MerkleTree::new();
        tree.add_leaf(b"data1");
        tree.add_leaf(b"data2");
        tree.add_leaf(b"data3");
        tree.add_leaf(b"data4");

        let proof = tree.generate_proof(0).unwrap();
        assert!(MerkleTree::verify_proof(&proof));
    }
}
