use sha2::{Sha256, Digest};
use std::fmt;

#[derive(Clone, Debug)]
pub struct UserData {
    pub user_id: u32,
    pub user_balance: u32,
}

impl UserData {
    fn new(user_id: u32, user_balance: u32) -> Self {
        UserData {
            user_id,
            user_balance,
        }
    }
}

#[derive(Clone)]
pub struct MerkleNode {
    hash: Vec<u8>,
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
    pub user_data: Option<UserData>,
}

impl MerkleNode {
    fn new_leaf(hash: Vec<u8>, user_data: Option<UserData>) -> Self {
        MerkleNode {
            hash,
            left: None,
            right: None,
            user_data,
        }
    }

    fn new_branch(left: MerkleNode, right: MerkleNode, tag: &str) -> Self {
        let combined = vec![left.hash.clone(), right.hash.clone()].concat();
        let hash = tagged_hash(tag, &combined);
        MerkleNode {
            hash,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            user_data: None,
        }
    }
}

impl fmt::Display for MerkleNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(user_data) = &self.user_data {
            write!(
                f,
                "{} (User ID: {}, Balance: {})",
                hex::encode(&self.hash),
                user_data.user_id,
                user_data.user_balance
            )
        } else {
            write!(f, "{}", hex::encode(&self.hash))
        }
    }
}

#[derive(Debug)]
pub struct TraversePath {
    pub hashes: Vec<String>,
    pub directions: Vec<u8>, // 0 for left, 1 for right
}

impl TraversePath {
    fn new() -> Self {
        TraversePath {
            hashes: Vec::new(),
            directions: Vec::new(),
        }
    }

    fn add_step(&mut self, hash: String, direction: u8) {
        self.hashes.push(hash);
        self.directions.push(direction);
    }

    pub fn to_vec(&self) -> Vec<(String, u8)> {
        self.hashes.iter().zip(self.directions.iter()).map(|(hash, direction)| {
            (hash.to_string(), direction.clone())
        }).collect()
    }
}

pub struct MerkleTree {
    root: Option<Box<MerkleNode>>,
}

impl MerkleTree {
    pub fn build(tag_leaf: &str, tag_branch: &str, user_data: &[(u32, u32)]) -> Self {
        if user_data.is_empty() {
            return MerkleTree { root: None };
        }

        let mut nodes: Vec<MerkleNode> = user_data
            .iter()
            .map(|&(user_id, user_balance)| {
                let user_data = UserData::new(user_id, user_balance);
                let serialized = format!("({},{})", user_id, user_balance);
                MerkleNode::new_leaf(tagged_hash(tag_leaf, serialized.as_bytes()), Some(user_data))
            })
            .collect();

        while nodes.len() > 1 {
            let mut next_level = Vec::new();

            for i in (0..nodes.len()).step_by(2) {
                let left = nodes[i].clone();
                let right = if i + 1 < nodes.len() {
                    nodes[i + 1].clone()
                } else {
                    nodes[i].clone()
                };

                let branch = MerkleNode::new_branch(left, right, tag_branch);
                next_level.push(branch);
            }

            nodes = next_level;
        }

        MerkleTree {
            root: Some(Box::new(nodes[0].clone())),
        }
    }

    pub fn root(&self) -> Option<String> {
        self.root.as_ref().map(|node|hex::encode( &node.hash))
    }

    fn print(&self) {
        if let Some(root) = &self.root {
            // Use a stack to simulate recursion
            let mut stack = Vec::new();
            stack.push((root, 0, "Root")); // (node, level, position)

            while let Some((node, level, position)) = stack.pop() {
                // Print the current node
                let indent = "  ".repeat(level);
                println!("{}{}: {}", indent, position, node);

                // Push the right child first (so the left child is processed first)
                if let Some(right) = &node.right {
                    stack.push((right, level + 1, "Right"));
                }
                if let Some(left) = &node.left {
                    stack.push((left, level + 1, "Left"));
                }
            }
        } else {
            println!("Tree is empty.");
        }
    }

    pub fn search_with_path<F>(&self, predicate: F) -> Option<(&MerkleNode, TraversePath)>
    where
        F: Fn(&UserData) -> bool,
    {
        if let Some(root) = &self.root {
            let mut path = TraversePath::new();
            Self::search_node_with_path(root, &predicate, &mut path)
        } else {
            None
        }
    }

    fn search_node_with_path<'a, F>(
        node: &'a MerkleNode,
        predicate: &F,
        path: &mut TraversePath,
    ) -> Option<(&'a MerkleNode, TraversePath)>
    where
        F: Fn(&UserData) -> bool,
    {
        if let Some(user_data) = &node.user_data {
            if predicate(user_data) {
                return Some((node, TraversePath {
                    directions: path.directions.clone(),
                    hashes: path.hashes.clone(),
                }));
            }
        }

        if let Some(left) = &node.left {
            path.add_step(hex::encode(&node.hash), 0); // 0 for left
            if let Some(result) = Self::search_node_with_path(left, predicate, path) {
                return Some(result);
            }
            path.hashes.pop(); // Backtrack
            path.directions.pop();
        }

        if let Some(right) = &node.right {
            path.add_step(hex::encode(&node.hash), 1); // 1 for right
            if let Some(result) = Self::search_node_with_path(right, predicate, path) {
                return Some(result);
            }
            path.hashes.pop(); // Backtrack
            path.directions.pop();
        }

        None
    }
}

fn tagged_hash(tag: &str, input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(tag.as_bytes());
    hasher.update(tag.as_bytes());
    hasher.update(input);
    hasher.finalize().to_vec()
}


#[cfg(test)]
mod tests {
    use rstest::rstest;

    #[rstest]
    #[case(
        "Bitcoin_Transaction",
        "aaa",
        "06d2aa1f6c35d838688c70c31592a6980ed28ece0766c83d232bae7caeed39e5"
    )]
    #[case(
        "Bitcoin_Transaction",
        "bbb",
        "fc97eea864d1343aeddb0daa35734c6d45b17b20b512e4290cf7f8ddd0bc1d50"
    )]
    #[case(
        "hello",
        "aaa",
        "a0dcb8268ac70eb9dca7342a36a1b3a8ba130b144bedb15bc9af5b2fa506a129"
    )]
    fn tagged_hash(#[case] tag: &str, #[case] input: &str, #[case] expected: &str) {
        let actual = super::tagged_hash(tag, input.as_bytes());
        assert_eq!(hex::encode(actual), expected);
    }

}
