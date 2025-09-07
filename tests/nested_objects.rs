//! Nested objects tests for link-validator

use link_validator::LinkValidator;
use serde_json::json;

#[test]
fn test_nested_object_fields() {
    let schema = json!({
        "user": {
            "type": "object",
            "required": true,
            "fields": {
                "name": {"type": "string", "required": true},
                "age": {"type": "integer", "min": 0}
            }
        }
    });

    let validator = LinkValidator::new(&schema).expect("Compilation failed");

    let data = json!({
        "user": {
            "name": "John",
            "age": 30
        }
    });

    let result = validator.validate(&data);
    assert!(result.is_valid);
}

#[test]
fn test_deeply_nested_object_fields() {
    let schema = json!({
        "user": {
            "type": "object",
            "required": true,
            "fields": {
                "profile": {
                    "type": "object",
                    "fields": {
                        "personal": {
                            "type": "object",
                            "fields": {
                                "name": {"type": "string", "required": true},
                                "age": {"type": "integer", "min": 0}
                            }
                        }
                    }
                }
            }
        }
    });

    let validator = LinkValidator::new(&schema).expect("Compilation failed");

    let data = json!({
        "user": {
            "profile": {
                "personal": {
                    "name": "John",
                    "age": 30
                }
            }
        }
    });

    let result = validator.validate(&data);
    assert!(result.is_valid);
}

#[test]
fn test_nested_array_fields() {
    let schema = json!({
        "users": {
            "type": "array",
            "required": true,
            "fields": {
                "user_item": {
                    "type": "object",
                    "fields": {
                        "name": {"type": "string", "required": true},
                        "age": {"type": "integer", "min": 0}
                    }
                }
            }
        }
    });

    let validator = LinkValidator::new(&schema).expect("Compilation failed");

    let data = json!({
        "users": [
            {
                "user_item": {
                    "name": "John",
                    "age": 30
                }
            },
            {
                "user_item": {
                    "name": "Jane",
                    "age": 25
                }
            }
        ]
    });

    let result = validator.validate(&data);
    assert!(result.is_valid);
}