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

#[test]
fn test_same_field_names_different_contexts() {
    // 测试 async-validator 中的 required (boolean) 和 JSON Schema 中的 required (array)
    let async_schema = json!({
        "name": {"type": "string", "required": true},
        "age": {"type": "number", "required": false}
    });
    
    let json_schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "number"}
        },
        "required": ["name"] // 注意这里 required 是数组
    });
    
    let async_validator = LinkValidator::new(&async_schema).expect("Async schema compilation failed");
    let json_validator = LinkValidator::new(&json_schema).expect("JSON schema compilation failed");
    
    // 测试缺少必需字段的情况
    let data = json!({
        "age": 25
        // 缺少 name 字段
    });
    
    let async_result = async_validator.validate(&data);
    let json_result = json_validator.validate(&data);
    
    // 两种 schema 都应该验证失败
    assert!(!async_result.is_valid);
    assert!(!json_result.is_valid);
    
    // 检查错误格式是否正确
    let async_errors = async_result.errors.as_array().unwrap();
    let json_errors = json_result.errors.as_array().unwrap();
    
    assert!(!async_errors.is_empty());
    assert!(!json_errors.is_empty());
    
    // Async-validator 错误应该有 "field" 字段
    for error in async_errors {
        assert!(error.get("field").is_some());
    }
    
    // JSON Schema 错误应该有 "instancePath" 字段
    for error in json_errors {
        assert!(error.get("instancePath").is_some());
    }
}

#[test]
fn test_min_max_field_differences() {
    // 测试 async-validator 中的 min/max 和 JSON Schema 中对应的 minLength/maxLength 等
    // 这个测试主要是验证我们的转换逻辑，而不是完整的验证功能
    
    let async_schema = json!({
        "text": {"type": "string", "min": 5, "max": 10}
    });
    
    let json_schema = json!({
        "type": "object",
        "properties": {
            "text": {"type": "string", "minLength": 5, "maxLength": 10}
        }
    });
    
    let async_validator = LinkValidator::new(&async_schema).expect("Async schema compilation failed");
    let json_validator = LinkValidator::new(&json_schema).expect("JSON schema compilation failed");
    
    // 两种格式都应该成功编译并创建验证器
    // 通过验证错误格式来判断 schema 类型
    let test_data = json!({"text": "hi"}); // 太短，会触发验证错误
    
    let async_result = async_validator.validate(&test_data);
    let json_result = json_validator.validate(&test_data);
    
    // 两种格式都应该检测到错误
    assert!(!async_result.is_valid);
    assert!(!json_result.is_valid);
    
    // 检查错误格式来区分 schema 类型
    let async_errors = async_result.errors.as_array().unwrap();
    let json_errors = json_result.errors.as_array().unwrap();
    
    // async-validator 格式的错误应该包含 "field" 字段
    for error in async_errors {
        assert!(error.get("field").is_some());
        assert!(error.get("instancePath").is_none());
    }
    
    // JSON Schema 格式的错误应该包含 "instancePath" 字段
    for error in json_errors {
        assert!(error.get("instancePath").is_some());
        assert!(error.get("field").is_none());
    }
}
