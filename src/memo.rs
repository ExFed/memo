// Unit tests for the memo module

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Memo {
    pub command: String,
    pub exit_code: i32,
    pub timestamp: String,
    pub digest: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memo_serialization() {
        let memo = Memo {
            command: "echo hello".to_string(),
            exit_code: 0,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: "abc123".to_string(),
        };

        let json = serde_json::to_string(&memo).unwrap();
        assert!(json.contains("echo hello"));
        assert!(json.contains("\"exit_code\":0"));
        assert!(json.contains("abc123"));
    }

    #[test]
    fn test_memo_deserialization() {
        let json = r#"{
            "command": "echo test",
            "exit_code": 42,
            "timestamp": "2025-12-22T01:51:52.369Z",
            "digest": "def456"
        }"#;

        let memo: Memo = serde_json::from_str(json).unwrap();
        assert_eq!(memo.command, "echo test");
        assert_eq!(memo.exit_code, 42);
        assert_eq!(memo.digest, "def456");
    }

    #[test]
    fn test_memo_roundtrip() {
        let original = Memo {
            command: "ls -la".to_string(),
            exit_code: 1,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: "xyz789".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Memo = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_memo_with_special_characters() {
        let memo = Memo {
            command: "echo \"hello\" 'world' $USER".to_string(),
            exit_code: 0,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: "special123".to_string(),
        };

        let json = serde_json::to_string(&memo).unwrap();
        let deserialized: Memo = serde_json::from_str(&json).unwrap();

        assert_eq!(memo.command, deserialized.command);
    }

    #[test]
    fn test_memo_negative_exit_code() {
        let memo = Memo {
            command: "test".to_string(),
            exit_code: -1,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: "neg123".to_string(),
        };

        let json = serde_json::to_string(&memo).unwrap();
        let deserialized: Memo = serde_json::from_str(&json).unwrap();

        assert_eq!(memo.exit_code, deserialized.exit_code);
    }

    #[test]
    fn test_memo_multiline_command() {
        let memo = Memo {
            command: "echo hello\necho world".to_string(),
            exit_code: 0,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: "multi123".to_string(),
        };

        let json = serde_json::to_string(&memo).unwrap();
        let deserialized: Memo = serde_json::from_str(&json).unwrap();

        assert_eq!(memo.command, deserialized.command);
    }
}
