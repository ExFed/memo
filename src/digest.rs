// Unit tests for the digest module

use sha2::{Digest, Sha256};

#[cfg(test)]
pub fn compute_digest(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn compute_digest_for_args(args: &[String]) -> Result<String, serde_json::Error> {
    // Hash a canonical encoding of argv to avoid collisions like:
    // ["echo", "a b"] vs ["echo", "a", "b"].
    let encoded = serde_json::to_vec(args)?;
    let mut hasher = Sha256::new();
    hasher.update(&encoded);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_same_input_same_output() {
        let digest1 = compute_digest("echo hello");
        let digest2 = compute_digest("echo hello");
        assert_eq!(digest1, digest2);
    }

    #[test]
    fn test_digest_different_input_different_output() {
        let digest1 = compute_digest("echo hello");
        let digest2 = compute_digest("echo world");
        assert_ne!(digest1, digest2);
    }

    #[test]
    fn test_digest_format() {
        let digest = compute_digest("test");
        // SHA-256 produces 64 hex characters
        assert_eq!(digest.len(), 64);
        // Should only contain hex characters
        assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_digest_whitespace_sensitive() {
        let digest1 = compute_digest("echo hello");
        let digest2 = compute_digest("echo  hello"); // two spaces
        assert_ne!(digest1, digest2);
    }

    #[test]
    fn test_digest_order_sensitive() {
        let digest1 = compute_digest("echo hello world");
        let digest2 = compute_digest("echo world hello");
        assert_ne!(digest1, digest2);
    }

    #[test]
    fn test_digest_empty_string() {
        let digest = compute_digest("");
        assert_eq!(digest.len(), 64);
        // SHA-256 of empty string
        assert_eq!(
            digest,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_digest_special_characters() {
        let digest1 = compute_digest("echo \"hello\" 'world' $USER");
        let digest2 = compute_digest("echo \"hello\" 'world' $USER");
        assert_eq!(digest1, digest2);
    }

    #[test]
    fn test_digest_multiline() {
        let digest1 = compute_digest("echo hello\necho world");
        let digest2 = compute_digest("echo hello\necho world");
        assert_eq!(digest1, digest2);
    }

    #[test]
    fn test_digest_known_value() {
        // Test against a known SHA-256 value
        let digest = compute_digest("hello");
        assert_eq!(
            digest,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_digest_args_avoids_join_collisions() {
        let a = vec!["echo".to_string(), "a b".to_string()];
        let b = vec!["echo".to_string(), "a".to_string(), "b".to_string()];

        let da = compute_digest_for_args(&a).unwrap();
        let db = compute_digest_for_args(&b).unwrap();
        assert_ne!(da, db);
    }
}
