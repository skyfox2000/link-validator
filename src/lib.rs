//! link-validate - async-validator 风格规则到 JSON Schema 转换器和验证器
//! 
//! 这个库提供将 async-validator 风格的验证规则转换为 JSON Schema 的功能，
//! 并使用 JSON Schema 进行数据验证。

use serde::{Deserialize, Serialize};
use serde_json::{Value, Map};
use jsonschema::JSONSchema;
use std::collections::HashMap;

/// Schema 格式类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SchemaFormat {
    /// JSON Schema 格式
    JsonSchema,
    /// Async-validator 规则格式
    AsyncValidator,
}

/// LinkValidator 验证器，包含编译后的schema和原始格式信息
#[derive(Debug)]
pub struct LinkValidator {
    /// 编译后的 JSON Schema
    schema: JSONSchema,
    /// 原始 schema 的格式类型
    format: SchemaFormat,
}

/// 验证结果
#[derive(Debug)]
pub struct ValidationResult {
    /// 验证是否通过
    pub is_valid: bool,
    /// 错误信息（JSON 格式）
    pub errors: Value,
}

impl LinkValidator {
    /// 使用当前验证器验证数据
    /// 
    /// # 参数
    /// 
    /// * `data` - 要验证的数据（JSON 格式）
    /// 
    /// # 返回值
    /// 
    /// 返回 ValidationResult 结构体，包含验证结果和错误信息
    pub fn validate(&self, data: &Value) -> ValidationResult {
        match self.schema.validate(data) {
            Ok(_) => ValidationResult {
                is_valid: true,
                errors: Value::Array(vec![]),
            },
            Err(errors) => {
                if self.format == SchemaFormat::AsyncValidator {
                    // 转换为 async-validator 错误格式
                    let error_messages: Vec<Value> = errors.into_iter().map(|e| {
                        serde_json::json!({
                            "message": e.to_string(),
                            "field": e.instance_path.to_string()
                        })
                    }).collect();
                    
                    ValidationResult {
                        is_valid: false,
                        errors: Value::Array(error_messages),
                    }
                } else {
                    // 保持 JSON Schema 错误格式
                    let error_messages: Vec<Value> = errors.into_iter().map(|e| {
                        serde_json::json!({
                            "message": e.to_string(),
                            "instancePath": e.instance_path.to_string()
                        })
                    }).collect();
                    
                    ValidationResult {
                        is_valid: false,
                        errors: Value::Array(error_messages),
                    }
                }
            }
        }
    }
}

// 内部结构，不对外公开
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AsyncValidatorRule {
    /// 字段类型
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    field_type: Option<String>,
    
    /// 是否必填
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<bool>,
    
    /// 最小长度（字符串）或最小值（数字）
    #[serde(skip_serializing_if = "Option::is_none")]
    min: Option<Value>,
    
    /// 最大长度（字符串）或最大值（数字）
    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<Value>,
    
    /// 精确长度
    #[serde(skip_serializing_if = "Option::is_none")]
    len: Option<Value>,
    
    /// 正则表达式模式
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern: Option<String>,
    
    /// 枚举值
    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    enum_values: Option<Vec<Value>>,
    
    /// 错误消息
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    
    /// 是否检查空白字符
    #[serde(skip_serializing_if = "Option::is_none")]
    whitespace: Option<bool>,
    
    /// 字段验证器（不支持转换）
    #[serde(skip_serializing_if = "Option::is_none")]
    validator: Option<Value>,
    
    /// 异步字段验证器（不支持转换）
    #[serde(skip_serializing_if = "Option::is_none")]
    async_validator: Option<Value>,
    
    /// 触发方式（不支持转换）
    #[serde(skip_serializing_if = "Option::is_none")]
    trigger: Option<Value>,
    
    /// 嵌套字段规则（用于支持深度嵌套）
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Value>,
    
    /// 其他未映射的属性
    #[serde(flatten)]
    extra: Map<String, Value>,
}

// 内部类型别名
type AsyncValidatorRules = HashMap<String, Vec<AsyncValidatorRule>>;

/// 编译 schema，返回 LinkValidator 验证器
/// 
/// 该函数会自动检测 schema 格式（JSON Schema 或 async-validator 规则），
/// 如果是 async-validator 规则格式，会自动转换为 JSON Schema 并编译。
/// 
/// # 参数
/// 
/// * `schema` - 要编译的 schema（JSON 格式），可以是 JSON Schema 或 async-validator 规则格式
/// 
/// # 返回值
/// 
/// 返回 LinkValidator 验证器，包含编译后的 schema 和原始格式信息
pub fn compile(schema: &Value) -> Result<LinkValidator, String> {
    // 判断是否为 async-validator 规则格式
    if is_async_rules(schema) {
        // 如果是 async-validator 规则，则需要转换
        match parse_async_rules(schema) {
            Ok(rules) => {
                match convert_to_jsonschema(&rules) {
                    Ok(conversion_result) => {
                        // 输出不支持的规则警告
                        for unsupported in &conversion_result.unsupported {
                            eprintln!("Warning: {}", unsupported);
                        }
                        
                        // 编译转换后的 schema
                        match JSONSchema::compile(&conversion_result.schema) {
                            Ok(compiled_schema) => {
                                Ok(LinkValidator {
                                    schema: compiled_schema,
                                    format: SchemaFormat::AsyncValidator,
                                })
                            },
                            Err(e) => {
                                Err(format!("Failed to compile converted schema: {}", e))
                            }
                        }
                    },
                    Err(e) => {
                        Err(format!("Failed to convert async-validator rules: {}", e))
                    }
                }
            },
            Err(e) => {
                Err(format!("Failed to parse async-validator rules: {}", e))
            }
        }
    } else {
        // 否则直接编译
        match JSONSchema::compile(schema) {
            Ok(compiled_schema) => {
                Ok(LinkValidator {
                    schema: compiled_schema,
                    format: SchemaFormat::JsonSchema,
                })
            },
            Err(e) => {
                Err(format!("Failed to compile schema: {}", e))
            }
        }
    }
}

/// 使用预编译的 schema 验证数据
/// 
/// # 参数
/// 
/// * `compiled` - 通过 [compile] 函数编译得到的结果
/// * `data` - 要验证的数据（JSON 格式）
/// 
/// # 返回值
/// 
/// 返回 ValidationResult 结构体，包含验证结果和错误信息
pub fn check(compiled: &CompileResult, data: &Value) -> ValidationResult {
    match compiled.schema.validate(data) {
        Ok(_) => ValidationResult {
            is_valid: true,
            errors: Value::Array(vec![]),
        },
        Err(errors) => {
            if compiled.format == SchemaFormat::AsyncValidator {
                // 转换为 async-validator 错误格式
                let error_messages: Vec<Value> = errors.into_iter().map(|e| {
                    serde_json::json!({
                        "message": e.to_string(),
                        "field": e.instance_path.to_string()
                    })
                }).collect();
                
                ValidationResult {
                    is_valid: false,
                    errors: Value::Array(error_messages),
                }
            } else {
                // 保持 JSON Schema 错误格式
                let error_messages: Vec<Value> = errors.into_iter().map(|e| {
                    serde_json::json!({
                        "message": e.to_string(),
                        "instancePath": e.instance_path.to_string()
                    })
                }).collect();
                
                ValidationResult {
                    is_valid: false,
                    errors: Value::Array(error_messages),
                }
            }
        }
    }
}

/// 验证数据的唯一公共接口
/// 
/// 该函数会自动检测 schema 格式（JSON Schema 或 async-validator 规则），
/// 如果是 async-validator 规则格式，会自动转换为 JSON Schema，
/// 然后编译 schema 并验证数据。
/// 
/// 注意：如果需要多次验证，建议使用 [compile] 和 [validate_with_compiled] 函数，
/// 以避免重复编译 schema。
/// 
/// # 参数
/// 
/// * `schema` - 要编译的 schema（JSON 格式），可以是 JSON Schema 或 async-validator 规则格式
/// * `data` - 要验证的数据（JSON 格式）
/// 
/// # 返回值
/// 
/// 返回 ValidationResult 结构体，包含验证结果和错误信息
/// 
/// # 示例
/// 
/// ```
/// use link_validate::validate;
/// use serde_json::json;
/// 
/// let schema = json!({
///     "username": {"type": "string", "required": true, "min": 3},
///     "email": {"type": "email", "required": true}
/// });
/// 
/// let data = json!({
///     "username": "john_doe",
///     "email": "john@example.com"
/// });
/// 
/// let result = validate(&schema, &data);
/// assert!(result.is_valid);
/// ```
pub fn validate(schema: &Value, data: &Value) -> ValidationResult {
    match compile(schema) {
        Ok(compiled) => validate_with_compiled(&compiled, data),
        Err(e) => ValidationResult {
            is_valid: false,
            errors: Value::Array(vec![Value::String(e)]),
        }
    }
}

/// 内部验证函数
fn validate_data_internal(schema: &JSONSchema, data: &Value, is_async_format: bool) -> ValidationResult {
    match schema.validate(data) {
        Ok(_) => ValidationResult {
            is_valid: true,
            errors: Value::Array(vec![]),
        },
        Err(errors) => {
            if is_async_format {
                // 转换为 async-validator 错误格式
                let error_messages: Vec<Value> = errors.into_iter().map(|e| {
                    serde_json::json!({
                        "message": e.to_string(),
                        "field": e.instance_path.to_string()
                    })
                }).collect();
                
                ValidationResult {
                    is_valid: false,
                    errors: Value::Array(error_messages),
                }
            } else {
                // 保持 JSON Schema 错误格式
                let error_messages: Vec<Value> = errors.into_iter().map(|e| {
                    serde_json::json!({
                        "message": e.to_string(),
                        "instancePath": e.instance_path.to_string()
                    })
                }).collect();
                
                ValidationResult {
                    is_valid: false,
                    errors: Value::Array(error_messages),
                }
            }
        }
    }
}

/// 判断给定的值是否为 async-validator 规则格式
fn is_async_rules(value: &Value) -> bool {
    // 简单检查是否为 async-validator 规则格式
    // async-validator 规则通常是对象，其中值是规则对象或规则对象数组
    match value {
        Value::Object(obj) => {
            // 检查是否至少有一个字段
            for (_, v) in obj {
                match v {
                    // 规则可以是单个对象
                    Value::Object(_) => return true,
                    // 规则可以是对象数组
                    Value::Array(arr) => {
                        if !arr.is_empty() {
                            if let Value::Object(_) = &arr[0] {
                                return true;
                            }
                        }
                    }
                    _ => continue,
                }
            }
            false
        },
        _ => false,
    }
}

/// 解析 async-validator 规则，支持对象和数组两种格式
fn parse_async_rules(value: &Value) -> Result<AsyncValidatorRules, Box<dyn std::error::Error>> {
    let mut rules = HashMap::new();
    
    if let Value::Object(obj) = value {
        for (field_name, field_rules) in obj {
            match field_rules {
                // 单个规则对象格式 { field: {type: "string", required: true} }
                Value::Object(rule_obj) => {
                    let rule: AsyncValidatorRule = serde_json::from_value(Value::Object(rule_obj.clone()))
                        .map_err(|e| format!("Failed to parse rule for field '{}': {}", field_name, e))?;
                    rules.insert(field_name.clone(), vec![rule]);
                },
                // 规则数组格式 { field: [{type: "string", required: true}, {min: 3}] }
                Value::Array(rule_arr) => {
                    let mut parsed_rules = Vec::new();
                    for (index, rule_value) in rule_arr.iter().enumerate() {
                        if let Value::Object(rule_obj) = rule_value {
                            let rule: AsyncValidatorRule = serde_json::from_value(Value::Object(rule_obj.clone()))
                                .map_err(|e| format!("Failed to parse rule {} for field '{}': {}", index, field_name, e))?;
                            parsed_rules.push(rule);
                        } else {
                            return Err(format!("Rule {} for field '{}' is not an object", index, field_name).into());
                        }
                    }
                    rules.insert(field_name.clone(), parsed_rules);
                },
                _ => {
                    return Err(format!("Invalid rule format for field '{}'", field_name).into());
                }
            }
        }
        Ok(rules)
    } else {
        Err("Input is not an object".into())
    }
}

/// 将 async-validator 规则转换为 JSON Schema
fn convert_to_jsonschema(rules: &AsyncValidatorRules) -> Result<ConversionResult, Box<dyn std::error::Error>> {
    let mut schema_object = Map::new();
    schema_object.insert("type".to_string(), Value::String("object".to_string()));
    
    let mut properties = Map::new();
    let mut required = Vec::new();
    let mut unsupported = Vec::new();
    
    for (field_name, field_rules) in rules {
        let mut field_schema = Map::new();
        let mut field_required = false;
        
        for rule in field_rules {
            // 处理 type 规则
            if let Some(ref type_name) = rule.field_type {
                match type_name.as_str() {
                    "string" => {
                        field_schema.insert("type".to_string(), Value::String("string".to_string()));
                    }
                    "number" => {
                        field_schema.insert("type".to_string(), Value::String("number".to_string()));
                    }
                    "integer" => {
                        field_schema.insert("type".to_string(), Value::String("integer".to_string()));
                    }
                    "boolean" => {
                        field_schema.insert("type".to_string(), Value::String("boolean".to_string()));
                    }
                    "array" => {
                        field_schema.insert("type".to_string(), Value::String("array".to_string()));
                        // 处理嵌套数组项规则
                        if let Some(ref nested_fields) = rule.fields {
                            let nested_rules = parse_async_rules(nested_fields)?;
                            let nested_conversion = convert_to_jsonschema(&nested_rules)?;
                            field_schema.insert("items".to_string(), nested_conversion.schema);
                            unsupported.extend(nested_conversion.unsupported);
                        }
                    }
                    "object" => {
                        field_schema.insert("type".to_string(), Value::String("object".to_string()));
                        // 处理嵌套对象的 fields
                        if let Some(ref nested_fields) = rule.fields {
                            let nested_rules = parse_async_rules(nested_fields)?;
                            let nested_conversion = convert_to_jsonschema(&nested_rules)?;
                            field_schema.insert("properties".to_string(), nested_conversion.schema["properties"].clone());
                            if nested_conversion.schema.get("required").is_some() {
                                field_schema.insert("required".to_string(), nested_conversion.schema["required"].clone());
                            }
                            unsupported.extend(nested_conversion.unsupported);
                        }
                    }
                    "method" => {
                        field_schema.insert("type".to_string(), Value::String("object".to_string()));
                        field_schema.insert("instanceof".to_string(), Value::String("Function".to_string()));
                    }
                    "regexp" => {
                        field_schema.insert("type".to_string(), Value::String("string".to_string()));
                        // 注意：JSON Schema 没有内置的正则表达式类型验证
                    }
                    "date" => {
                        field_schema.insert("type".to_string(), Value::String("string".to_string()));
                        field_schema.insert("format".to_string(), Value::String("date-time".to_string()));
                    }
                    "email" => {
                        field_schema.insert("type".to_string(), Value::String("string".to_string()));
                        field_schema.insert("format".to_string(), Value::String("email".to_string()));
                    }
                    "url" => {
                        field_schema.insert("type".to_string(), Value::String("string".to_string()));
                        field_schema.insert("format".to_string(), Value::String("uri".to_string()));
                    }
                    "hex" => {
                        field_schema.insert("type".to_string(), Value::String("string".to_string()));
                        // 可以添加 pattern 来验证十六进制格式
                        field_schema.insert("pattern".to_string(), Value::String("^[0-9a-fA-F]+$".to_string()));
                    }
                    "any" => {
                        // JSON Schema 中没有 "any" 类型，使用 "type" 数组或者不指定类型
                        // 这里我们选择不指定类型（即允许任何类型）
                    }
                    _ => {
                        unsupported.push(format!("Field '{}': unsupported type '{}'", field_name, type_name));
                    }
                }
            }
            
            // 处理 required 规则
            if let Some(true) = rule.required {
                field_required = true;
            }
            
            // 处理 min 规则
            if let Some(ref min_value) = rule.min {
                match field_schema.get("type").and_then(|v| v.as_str()) {
                    Some("string") => {
                        field_schema.insert("minLength".to_string(), min_value.clone());
                    }
                    Some("array") => {
                        field_schema.insert("minItems".to_string(), min_value.clone());
                    }
                    Some("number") | Some("integer") => {
                        field_schema.insert("minimum".to_string(), min_value.clone());
                    }
                    _ => {
                        // 默认当作数值处理
                        field_schema.insert("minimum".to_string(), min_value.clone());
                    }
                }
            }
            
            // 处理 max 规则
            if let Some(ref max_value) = rule.max {
                match field_schema.get("type").and_then(|v| v.as_str()) {
                    Some("string") => {
                        field_schema.insert("maxLength".to_string(), max_value.clone());
                    }
                    Some("array") => {
                        field_schema.insert("maxItems".to_string(), max_value.clone());
                    }
                    Some("number") | Some("integer") => {
                        field_schema.insert("maximum".to_string(), max_value.clone());
                    }
                    _ => {
                        // 默认当作数值处理
                        field_schema.insert("maximum".to_string(), max_value.clone());
                    }
                }
            }
            
            // 处理 len 规则
            if let Some(ref len_value) = rule.len {
                match field_schema.get("type").and_then(|v| v.as_str()) {
                    Some("string") => {
                        field_schema.insert("minLength".to_string(), len_value.clone());
                        field_schema.insert("maxLength".to_string(), len_value.clone());
                    }
                    Some("array") => {
                        field_schema.insert("minItems".to_string(), len_value.clone());
                        field_schema.insert("maxItems".to_string(), len_value.clone());
                    }
                    _ => {
                        unsupported.push(format!("Field '{}': len rule only supported for string and array types", field_name));
                    }
                }
            }
            
            // 处理 pattern 规则
            if let Some(ref pattern) = rule.pattern {
                field_schema.insert("pattern".to_string(), Value::String(pattern.clone()));
            }
            
            // 处理 enum 规则
            if let Some(ref enum_values) = rule.enum_values {
                field_schema.insert("enum".to_string(), Value::Array(enum_values.clone()));
            }
            
            // 处理 whitespace 规则
            if rule.whitespace.is_some() {
                // whitespace 规则需要自定义验证，JSON Schema 不直接支持
                unsupported.push(format!("Field '{}': whitespace rule not supported in JSON Schema", field_name));
            }
            
            // 检查不支持的规则
            if rule.validator.is_some() {
                unsupported.push(format!("Field '{}': validator function not supported", field_name));
            }
            
            if rule.async_validator.is_some() {
                unsupported.push(format!("Field '{}': asyncValidator function not supported", field_name));
            }
            
            if rule.trigger.is_some() {
                unsupported.push(format!("Field '{}': trigger option not supported", field_name));
            }
            
            for (key, _) in &rule.extra {
                match key.as_str() {
                    "validator" | "asyncValidator" | "trigger" | "whitespace" | "transform" | "fields" => {
                        // 已经处理过这些规则
                    }
                    _ => {
                        unsupported.push(format!("Field '{}': unsupported rule '{}'", field_name, key));
                    }
                }
            }
        }
        
        // 如果没有指定类型，默认为字符串
        if !field_schema.contains_key("type") && rule.field_type.is_some() {
            field_schema.insert("type".to_string(), Value::String("string".to_string()));
        }
        
        properties.insert(field_name.clone(), Value::Object(field_schema));
        
        if field_required {
            required.push(field_name.clone());
        }
    }
    
    schema_object.insert("properties".to_string(), Value::Object(properties));
    
    if !required.is_empty() {
        schema_object.insert("required".to_string(), Value::Array(
            required.into_iter().map(Value::String).collect()
        ));
    }
    
    let schema = Value::Object(schema_object);
    
    Ok(ConversionResult {
        schema,
        unsupported,
    })
}

/// 转换结果（内部使用）
#[derive(Debug)]
struct ConversionResult {
    /// 生成的 JSON Schema
    schema: Value,
    /// 不支持的验证规则列表
    unsupported: Vec<String>,
}

impl Default for AsyncValidatorRule {
    fn default() -> Self {
        AsyncValidatorRule {
            field_type: None,
            required: None,
            min: None,
            max: None,
            len: None,
            pattern: None,
            enum_values: None,
            message: None,
            whitespace: None,
            validator: None,
            async_validator: None,
            trigger: None,
            extra: Map::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_async_validator_rules() {
        let schema = json!({
            "username": {"type": "string", "required": true, "min": 3},
            "email": {"type": "email", "required": true}
        });

        let validator = compile(&schema).expect("Compilation failed");
        assert_eq!(validator.format, SchemaFormat::AsyncValidator);

        let data = json!({
            "username": "john_doe",
            "email": "john@example.com"
        });

        let result = validator.validate(&data);
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

        let validator = compile(&schema).expect("Compilation failed");
        assert_eq!(validator.format, SchemaFormat::JsonSchema);

        let data = json!({
            "username": "john_doe",
            "email": "john@example.com"
        });

        let result = validator.validate(&data);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validation_failure() {
        let schema = json!({
            "username": {"type": "string", "required": true, "min": 3},
            "email": {"type": "email", "required": true}
        });

        let validator = compile(&schema).expect("Compilation failed");

        let data = json!({
            "username": "jo", // Too short
            "email": "invalid-email"
        });

        let result = validator.validate(&data);
        assert!(!result.is_valid);
        assert!(!result.errors.as_array().unwrap().is_empty());
    }

    #[test]
    fn test_enum_validation() {
        let schema = json!({
            "status": {"type": "string", "enum": ["active", "inactive", "pending"]}
        });

        let validator = compile(&schema).expect("Compilation failed");

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

        let validator = compile(&schema).expect("Compilation failed");

        let data = json!({
            "tags": ["rust", "web"]
        });

        let result = validator.validate(&data);
        assert!(result.is_valid);
    }

    #[test]
    fn test_nested_object_fields() {
        let schema = json!({
            "user": {
                "type": "object",
                "required": true,
                "fields": {
                    "name": {"type": "string", "required": true},
                    "age": {"type": "integer", "minimum": 0}
                }
            }
        });

        let validator = compile(&schema).expect("Compilation failed");

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
                                    "age": {"type": "integer", "minimum": 0}
                                }
                            }
                        }
                    }
                }
            }
        });

        let validator = compile(&schema).expect("Compilation failed");

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
                    "type": "object",
                    "fields": {
                        "name": {"type": "string", "required": true},
                        "age": {"type": "integer", "minimum": 0}
                    }
                }
            }
        });

        let validator = compile(&schema).expect("Compilation failed");

        let data = json!({
            "users": [
                {
                    "name": "John",
                    "age": 30
                },
                {
                    "name": "Jane",
                    "age": 25
                }
            ]
        });

        let result = validator.validate(&data);
        assert!(result.is_valid);
    }

    #[test]
    fn test_compile_error() {
        let schema = json!("invalid schema");
        let result = compile(&schema);
        assert!(result.is_err());
    }
}
