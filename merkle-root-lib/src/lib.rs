use sha2::{Digest, Sha256};

fn tagged_hash(tag: &str, input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(tag.as_bytes());
    hasher.update(tag.as_bytes());
    hasher.update(input);
    hasher.finalize().to_vec()
}

pub fn compute(tag_branch: &str, tag_leaf: &str, input: &[String]) -> String {
    let mut hashes: Vec<Vec<u8>> = input
        .iter()
        .map(|item| tagged_hash(tag_leaf, item.as_bytes()))
        .collect::<Vec<Vec<u8>>>();

    while hashes.len() > 1 {
        hashes = hashes
            .chunks(2)
            .map(|chunk| {
                let left = &chunk[0];
                let right = chunk.get(1)
                    .unwrap_or(left);

                let combined = vec![left.clone(), right.clone()].concat();
                tagged_hash(tag_branch, &combined)
            })
            .collect()
    }

    let merkle_root = &hashes[0];
    hex::encode(merkle_root)
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

    #[rstest]
    #[case(
        "Bitcoin_Transaction",
        "Bitcoin_Transaction",
        "03c310cf2ad354009c474d1471c535065138ad1c976567451b7787a1983af425"
    )]
    #[case(
        "ProofOfReserve_Branch",
        "ProofOfReserve_Leaf",
        "61e782c764bbdc8f705a5d9db4c28a54eed85b6f047d85a08649bd40404b3495"
    )]
    fn compute(#[case] tag_branch: &str, #[case] tag_leaf: &str, #[case] expected: &str) {
        let input = vec![
            "aaa".to_string(),
            "bbb".to_string(),
            "ccc".to_string(),
            "ddd".to_string(),
            "eee".to_string(),
        ];

        let merkle_root = super::compute(tag_branch, tag_leaf, &input);

        assert_eq!(merkle_root, expected);
    }
}
