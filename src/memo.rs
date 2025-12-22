// Unit tests for the memo module

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Memo {
    pub cmd: Vec<String>,
    pub exit_code: i32,
    pub timestamp: String,
    pub digest: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ts() -> String {
        "2025-12-22T01:51:52.369Z".to_string()
    }

    #[test]
    fn test_memo_serialization() {
        let memo = Memo {
            cmd: vec!["echo".to_string(), "hello".to_string()],
            exit_code: 0,
            timestamp: ts(),
            digest: "abc123".to_string(),
        };

        let json = serde_json::to_string(&memo).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["cmd"], json!(["echo", "hello"]));
        assert_eq!(value["exit_code"], json!(0));
        assert_eq!(value["digest"], json!("abc123"));
    }

    #[test]
    fn test_memo_deserialization() {
        let json = r#"{
            "cmd": ["echo", "test"],
            "exit_code": 42,
            "timestamp": "2025-12-22T01:51:52.369Z",
            "digest": "def456"
        }"#;

        let memo: Memo = serde_json::from_str(json).unwrap();
        assert_eq!(memo.cmd, vec!["echo", "test"]);
        assert_eq!(memo.exit_code, 42);
        assert_eq!(memo.digest, "def456");
    }

    #[test]
    fn test_memo_roundtrip() {
        let original = Memo {
            cmd: vec!["ls".to_string(), "-la".to_string()],
            exit_code: 1,
            timestamp: ts(),
            digest: "xyz789".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Memo = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_memo_with_special_characters() {
        let memo = Memo {
            cmd: vec!["echo".to_string(), "\"hello\" 'world' $USER".to_string()],
            exit_code: 0,
            timestamp: ts(),
            digest: "special123".to_string(),
        };

        let json = serde_json::to_string(&memo).unwrap();
        let deserialized: Memo = serde_json::from_str(&json).unwrap();

        assert_eq!(memo.cmd, deserialized.cmd);
    }

    #[test]
    fn test_memo_negative_exit_code() {
        let memo = Memo {
            cmd: vec!["test".to_string()],
            exit_code: -1,
            timestamp: ts(),
            digest: "neg123".to_string(),
        };

        let json = serde_json::to_string(&memo).unwrap();
        let deserialized: Memo = serde_json::from_str(&json).unwrap();

        assert_eq!(memo.exit_code, deserialized.exit_code);
    }

    #[test]
    fn test_memo_multiline_command() {
        let memo = Memo {
            cmd: vec!["sh".to_string(), "-c".to_string(), "echo hello\necho world".to_string()],
            exit_code: 0,
            timestamp: ts(),
            digest: "multi123".to_string(),
        };

        let json = serde_json::to_string(&memo).unwrap();
        let deserialized: Memo = serde_json::from_str(&json).unwrap();

        assert_eq!(memo.cmd, deserialized.cmd);
    }
}
