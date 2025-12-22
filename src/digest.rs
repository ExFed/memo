// Unit tests for the digest module

use sha2::{Digest, Sha256};

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
    use shell_words::split;

    fn digest_for_args(args: &[String]) -> String {
        compute_digest_for_args(args).expect("failed to compute digest")
    }

    fn digest_for_command(command: &str) -> String {
        let args = split(command).expect("failed to parse command");
        digest_for_args(&args)
    }

    #[test]
    fn test_digest_same_command_same_output() {
        let digest1 = digest_for_command("echo hello");
        let digest2 = digest_for_command("echo hello");
        assert_eq!(digest1, digest2);
    }

    #[test]
    fn test_digest_different_commands_different_output() {
        let digest1 = digest_for_command("echo hello");
        let digest2 = digest_for_command("echo world");
        assert_ne!(digest1, digest2);
    }

    #[test]
    fn test_digest_format() {
        let digest = digest_for_command("echo test");
        assert_eq!(digest.len(), 64);
        // Should only contain hex characters
        assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_digest_whitespace_collapses() {
        let digest1 = digest_for_command("echo   hello");
        let digest2 = digest_for_command("echo hello");
        assert_eq!(digest1, digest2);
    }

    #[test]
    fn test_digest_order_sensitive() {
        let digest1 = digest_for_args(&vec!["echo".into(), "hello".into(), "world".into()]);
        let digest2 = digest_for_args(&vec!["echo".into(), "world".into(), "hello".into()]);
        assert_ne!(digest1, digest2);
    }

    #[test]
    fn test_digest_empty_args_known_value() {
        let digest = digest_for_command("");
        assert_eq!(digest.len(), 64);
        assert_eq!(digest, "4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945");
    }

    #[test]
    fn test_digest_quoting_changes_arguments() {
        let quoted = digest_for_command("echo 'hello world'");
        let unquoted = digest_for_command("echo hello world");
        assert_ne!(quoted, unquoted);
    }

    #[test]
    fn test_digest_args_avoids_join_collisions() {
        let quoted = digest_for_command("echo 'a b'");
        let split_args = digest_for_command("echo a b");
        assert_ne!(quoted, split_args);
    }

    #[test]
    fn test_digest_known_value_for_echo_hello() {
        let digest = digest_for_args(&vec!["echo".into(), "hello".into()]);

        assert_eq!(
            digest,
            "8b7f749f4aa4672a7feec75fdc79f1fb5aa71f8177a667b94e89a92c9a54714b"
        );
    }

    #[test]
    fn test_digest_special_characters_are_preserved() {
        let digest1 = digest_for_command("echo \"hello\" 'world' $USER");
        let digest2 = digest_for_command("echo \"hello\" 'world' $USER");
        assert_eq!(digest1, digest2);
    }
}
