use sha2::{Digest, Sha256};
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
    left: Option<usize>,
    right: Option<usize>,
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
}

impl MerkleTree {
    fn new_branch(&mut self, left: usize, right: usize, tag: &str) -> usize {
        let combined = vec![
            self.nodes[left].hash.clone(),
            self.nodes[right].hash.clone(),
        ]
        .concat();
        let hash = tagged_hash(tag, &combined);
        let new_node = MerkleNode {
            hash,
            left: Some(left),
            right: Some(right),
            user_data: None,
        };
        let ret = self.nodes.len();
        self.nodes.push(new_node);
        ret
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

#[derive(Debug, Clone)]
pub enum NodeDirection {
    Left,
    Right,
}

impl NodeDirection {
    fn value(&self) -> u8 {
        match self {
            NodeDirection::Left => 0,
            NodeDirection::Right => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraversePath {
    pub hashes: Vec<String>,
    pub directions: Vec<NodeDirection>,
}

impl TraversePath {
    fn new() -> Self {
        TraversePath {
            hashes: Vec::new(),
            directions: Vec::new(),
        }
    }

    fn add_step(&mut self, hash: String, direction: NodeDirection) {
        self.hashes.push(hash);
        self.directions.push(direction);
    }

    pub fn to_vec(&self) -> Vec<(String, u8)> {
        self.hashes
            .iter()
            .zip(self.directions.iter())
            .map(|(hash, direction)| (hash.to_string(), direction.value()))
            .collect()
    }
}

pub struct MerkleTree {
    root: Option<usize>,
    nodes: Vec<MerkleNode>,
}

impl MerkleTree {
    pub fn build(tag_leaf: &str, tag_branch: &str, user_data: &[(u32, u32)]) -> Self {
        if user_data.is_empty() {
            return MerkleTree {
                root: None,
                nodes: vec![],
            };
        }

        let nodes = user_data
            .iter()
            .map(|&(user_id, user_balance)| {
                let user_data = UserData::new(user_id, user_balance);
                let serialized = format!("({},{})", user_id, user_balance);
                MerkleNode::new_leaf(
                    tagged_hash(tag_leaf, serialized.as_bytes()),
                    Some(user_data),
                )
            })
            .collect();

        let mut tree = Self { root: None, nodes };

        let mut start = 0;

        while tree.nodes.len() - start > 1 {
            let next_start = tree.nodes.len();
            for i in (start..tree.nodes.len()).step_by(2) {
                let left = i;
                let right = (i + 1).min(next_start - 1);

                tree.new_branch(left, right, tag_branch);
            }
            start = next_start;
        }

        tree.root = Some(tree.nodes.len() - 1);
        tree
    }

    pub fn root(&self) -> Option<String> {
        self.root.map(|node| hex::encode(&self.nodes[node].hash))
    }

    fn print(&self) {
        if let Some(root) = &self.root {
            let mut stack = Vec::new();
            stack.push((root, 0, "Root")); // (node, level, position)

            while let Some((node, level, position)) = stack.pop() {
                let indent = "  ".repeat(level);
                println!("{}{}: {}", indent, position, self.nodes[*node]);

                if let Some(right) = &self.nodes[*node].right {
                    stack.push((right, level + 1, "Right"));
                }

                if let Some(left) = &self.nodes[*node].left {
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
            self.search_node_with_path(&self.nodes[*root], &predicate, &mut path)
        } else {
            None
        }
    }

    fn search_node_with_path<'a, F>(
        &'a self,
        node: &'a MerkleNode,
        predicate: &F,
        path: &mut TraversePath,
    ) -> Option<(&'a MerkleNode, TraversePath)>
    where
        F: Fn(&UserData) -> bool,
    {
        if let Some(user_data) = &node.user_data {
            if predicate(user_data) {
                return Some((
                    node,
                    TraversePath {
                        directions: path.directions.clone(),
                        hashes: path.hashes.clone(),
                    },
                ));
            }
        }

        if let Some(left) = &node.left {
            path.add_step(hex::encode(&node.hash), NodeDirection::Left); // 0 for left
            if let Some(result) = self.search_node_with_path(&self.nodes[*left], predicate, path) {
                return Some(result);
            }
            path.hashes.pop();
            path.directions.pop();
        }

        if let Some(right) = &node.right {
            path.add_step(hex::encode(&node.hash), NodeDirection::Right); // 1 for right
            if let Some(result) = self.search_node_with_path(&self.nodes[*right], predicate, path) {
                return Some(result);
            }
            path.hashes.pop();
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
    use super::*;
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

    #[test]
    fn it_can_build_a_tree() {
        let user_data = vec![(1, 1111), (2, 2222), (3, 3333), (4, 4444), (5, 5555)];
        let tag_leaf = "ProofOfReserve_Leaf";
        let tag_branch = "ProofOfReserve_Branch";

        let tree = MerkleTree::build(tag_leaf, tag_branch, &user_data);
        tree.print();

        assert_eq!(
            tree.root().unwrap(),
            "857f9bdfbbee9207675cbde460c99682015758111b8f9aad7193832619fb1782"
        );
    }

    #[test]
    fn it_can_search_with_path() {
        let user_data = vec![(1, 1111), (2, 2222), (3, 3333), (4, 4444), (5, 5555)];
        let tag_leaf = "ProofOfReserve_Leaf";
        let tag_branch = "ProofOfReserve_Branch";

        let tree = MerkleTree::build(tag_leaf, tag_branch, &user_data);
        let user_id = "3";
        let (node, path) = tree
            .search_with_path(|user_data| user_data.user_id == user_id.parse::<u32>().unwrap())
            .unwrap();

        assert_eq!(
            path.to_vec(),
            vec![
                (
                    "857f9bdfbbee9207675cbde460c99682015758111b8f9aad7193832619fb1782".to_string(),
                    0u8
                ),
                (
                    "09e1f208d3b96f4d5948225f3a1ea83fbc0017a80d1fcd2603ca537e958fcc57".to_string(),
                    1u8
                ),
                (
                    "76437464d68b779571e1d94270df86792faad0bdcfe2c0868459d4c9bd0ff5da".to_string(),
                    0u8
                )
            ]
        );
    }
}
