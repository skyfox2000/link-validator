//! Basic functionality tests for link-validator

use link_validator::LinkValidator;
use serde_json::json;

#[test]
fn test_basic_async_validator_rules() {
    let schema = json!({
        "username": {"type": "string", "required": true, "min": 3},
        "email": {"type": "email", "required": true}
    });

    let validator = LinkValidator::new(&schema).expect("Compilation failed");
    // We can indirectly test the format by checking the error format
    // Async-validator format should produce errors with "field" key

    let data = json!({
        "username": "jo", // Too short
        "email": "invalid-email"
    });

    let result = validator.validate(&data);
    assert!(!result.is_valid);
    
    // Check that errors follow async-validator format (field, not instancePath)
    let errors = result.errors.as_array().unwrap();
    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.get("field").is_some());
        assert!(error.get("instancePath").is_none());
    }
    
    // Test with valid data
    let valid_data = json!({
        "username": "john_doe",
        "email": "john@example.com"
    });

    let result = validator.validate(&valid_data);
    assert!(result.is_valid);
}

#[test]
fn test_json_schema_direct() {
    let schema = json!({
        "type": "object",
        "properties": {
            "username": {"type": "string", "minLength": 3},
            "email": {"type": "string", "format": "email"}
        },
        "required": ["username", "email"]
    });

    let validator = LinkValidator::new(&schema).expect("Compilation failed");
    // JSON Schema format should produce errors with "instancePath" key

    let data = json!({
        "username": "jo", // Too short
        "email": "invalid-email"
    });

    let result = validator.validate(&data);
    assert!(!result.is_valid);
    
    // Check that errors follow JSON Schema format (instancePath, not field)
    let errors = result.errors.as_array().unwrap();
    assert!(!errors.is_empty());
    for error in errors {
        assert!(error.get("instancePath").is_some());
        assert!(error.get("field").is_none());
    }
    
    // Test with valid data
    let valid_data = json!({
        "username": "john_doe",
        "email": "john@example.com"
    });

    let result = validator.validate(&valid_data);
    assert!(result.is_valid);
}

#[test]
fn test_validation_failure() {
    let schema = json!({
        "username": {"type": "string", "required": true, "min": 3},
        "email": {"type": "email", "required": true}
    });

    let validator = LinkValidator::new(&schema).expect("Compilation failed");

    let data = json!({
        "username": "jo", // Too short
        "email": "invalid-email"
    });

    let result = validator.validate(&data);
    assert!(!result.is_valid);
    assert!(!result.errors.as_array().unwrap().is_empty());
}