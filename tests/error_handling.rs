//! Error handling tests for link-validator

use link_validator::LinkValidator;
use serde_json::json;

#[test]
fn test_compile_error() {
    let schema = json!("invalid schema");
    let result = LinkValidator::new(&schema);
    assert!(result.is_err());
}

#[test]
fn test_invalid_schema_type() {
    let schema = json!(42);
    let result = LinkValidator::new(&schema);
    assert!(result.is_err());
}

#[test]
fn test_async_validator_error_format() {
    let schema = json!({
        "username": {"type": "string", "required": true, "min": 3}
    });

    let validator = LinkValidator::new(&schema).expect("Compilation failed");
    
    // Test with invalid data to trigger errors
    let data = json!({
        "username": "jo" // Too short
    });

    let result = validator.validate(&data);
    assert!(!result.is_valid);
    
    // Check that errors are in the async-validator format
    let errors = result.errors.as_array().unwrap();
    assert!(!errors.is_empty());
    
    // Each error should have a message and field
    for error in errors {
        assert!(error.get("message").is_some());
        assert!(error.get("field").is_some());
    }
}

#[test]
fn test_json_schema_error_format() {
    let schema = json!({
        "type": "object",
        "properties": {
            "username": {"type": "string", "minLength": 3}
        },
        "required": ["username"]
    });

    let validator = LinkValidator::new(&schema).expect("Compilation failed");
    
    // Test with invalid data to trigger errors
    let data = json!({
        "username": "jo" // Too short
    });

    let result = validator.validate(&data);
    assert!(!result.is_valid);
    
    // Check that errors are in the JSON Schema format
    let errors = result.errors.as_array().unwrap();
    assert!(!errors.is_empty());
    
    // Each error should have a message and instancePath
    for error in errors {
        assert!(error.get("message").is_some());
        assert!(error.get("instancePath").is_some());
    }
}