//! Special types and rules tests for link-validator

use link_validator::LinkValidator;
use serde_json::json;

#[test]
fn test_enum_validation() {
    let schema = json!({
        "status": {"type": "string", "enum": ["active", "inactive", "pending"]}
    });

    let validator = LinkValidator::new(&schema).expect("Compilation failed");

    let data = json!({
        "status": "active"
    });

    let result = validator.validate(&data);
    assert!(result.is_valid);
}

#[test]
fn test_array_rules_format() {
    let schema = json!({
        "tags": [
            {"type": "array", "required": true},
            {"min": 1, "max": 5}
        ]
    });

    let validator = LinkValidator::new(&schema).expect("Compilation failed");

    let data = json!({
        "tags": ["rust", "web"]
    });

    let result = validator.validate(&data);
    assert!(result.is_valid);
}

#[test]
fn test_special_types() {
    let schema = json!({
        "url_field": {"type": "url", "required": true},
        "date_field": {"type": "date", "required": true},
        "email_field": {"type": "email", "required": true}
    });

    let validator = LinkValidator::new(&schema).expect("Compilation failed");

    let data = json!({
        "url_field": "https://example.com",
        "date_field": "2023-01-01T00:00:00Z",
        "email_field": "test@example.com"
    });

    let result = validator.validate(&data);
    assert!(result.is_valid);
}

#[test]
fn test_unsupported_rules_warning() {
    // Capture stderr to check for warnings
    let schema = json!({
        "field_with_validator": {
            "type": "string",
            "validator": "some custom function"
        },
        "field_with_transform": {
            "type": "string", 
            "transform": "some transform function"
        }
    });

    // This should compile but output warnings
    let validator = LinkValidator::new(&schema).expect("Compilation failed");
    
    let data = json!({
        "field_with_validator": "test",
        "field_with_transform": "test"
    });

    let result = validator.validate(&data);
    assert!(result.is_valid);
    // Note: We can't easily test stderr output in this context
    // In a real test, we might use a testing framework that captures stderr
}