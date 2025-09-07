//! link-validator - async-validator 风格规则到 JSON Schema 转换器和验证器
//! 
//! 这个库提供将 async-validator 风格的验证规则转换为 JSON Schema 的功能，
//! 并使用 JSON Schema 进行数据验证。
//! 
//! ## 功能概述
//! 
//! 1. **自动格式检测**：自动检测输入的 schema 是 JSON Schema 还是 async-validator 规则格式
//! 2. **格式转换**：将 async-validator 规则转换为标准的 JSON Schema
//! 3. **数据验证**：使用 JSON Schema 验证数据
//! 4. **编译检查**：编译 schema 并检查有效性
//! 
//! ## 支持的转换规则
//! 
//! ### 基础类型转换
//! - `string` -> JSON Schema string 类型
//! - `number` -> JSON Schema number 类型
//! - `integer` -> JSON Schema integer 类型
//! - `boolean` -> JSON Schema boolean 类型
//! - `array` -> JSON Schema array 类型
//! - `object` -> JSON Schema object 类型
//! 
//! ### 特殊类型转换
//! - `method` -> JSON Schema object 类型（标记为 Function 实例）
//! - `regexp` -> JSON Schema string 类型
//! - `date` -> JSON Schema string 类型 + date-time format
//! - `email` -> JSON Schema string 类型 + email format
//! - `url` -> JSON Schema string 类型 + uri format
//! - `hex` -> JSON Schema string 类型 + hex pattern
//! - `any` -> JSON Schema 无类型限制
//! 
//! ### 验证规则转换
//! - `required` -> JSON Schema required 字段
//! - `min`/`max` -> 根据类型转换为 minLength/maxLength 或 minimum/maximum
//! - `len` -> 转换为 minLength 和 maxLength (字符串) 或 minItems/maxItems (数组)
//! - `pattern` -> JSON Schema pattern (正则表达式)
//! - `enum` -> JSON Schema enum (枚举值)
//! - `fields` -> JSON Schema properties (嵌套对象)
//! 
//! ### 不支持的规则
//! 以下规则不支持转换，会在转换时输出警告：
//! - `validator` (自定义验证函数)
//! - `asyncValidator` (异步验证函数)
//! - `trigger` (触发方式)
//! - `whitespace` (空白字符处理)
//! - `transform` (值转换)
//! 
//! ## 使用示例
//! 
//! ### 基本用法
//! 
//! ```
//! use link_validator::LinkValidator;
//! use serde_json::json;
//! 
//! let schema = json!({
//!     "username": {"type": "string", "required": true, "min": 3},
//!     "email": {"type": "email", "required": true}
//! });
//! 
//! // 使用 new 方法创建验证器
//! let validator = LinkValidator::new(&schema).unwrap();
//! 
//! // 使用验证器多次验证不同数据
//! let data1 = json!({"username": "john", "email": "john@example.com"});
//! let result1 = validator.validate(&data1);
//! 
//! let data2 = json!({"username": "jane", "email": "jane@example.com"});
//! let result2 = validator.validate(&data2);
//! 
//! assert!(result1.is_valid);
//! assert!(result2.is_valid);
//! ```
//! 
//! ### 深度嵌套对象验证
//! 
//! ```
//! use link_validator::LinkValidator;
//! use serde_json::json;
//! 
//! // 支持深度嵌套的对象验证
//! let schema = json!({
//!     "user": {
//!         "type": "object",
//!         "required": true,
//!         "fields": {
//!             "profile": {
//!                 "type": "object",
//!                 "fields": {
//!                     "personal": {
//!                         "type": "object",
//!                         "fields": {
//!                             "name": {"type": "string", "required": true},
//!                             "age": {"type": "integer", "min": 0}
//!                         }
//!                     }
//!                 }
//!             }
//!         }
//!     }
//! });
//! 
//! // 使用 new 方法创建验证器
//! let validator = LinkValidator::new(&schema).unwrap();
//! 
//! // 验证深度嵌套的数据
//! let data = json!({
//!     "user": {
//!         "profile": {
//!             "personal": {
//!                 "name": "John",
//!                 "age": 30
//!             }
//!         }
//!     }
//! });
//! 
//! let result = validator.validate(&data);
//! assert!(result.is_valid);
//! ```

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

impl LinkValidator {
    /// 创建一个新的 LinkValidator 实例
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
    /// 
    /// # 示例
    /// 
    /// ```
    /// use link_validator::LinkValidator;
    /// use serde_json::json;
    /// 
    /// let schema = json!({
    ///     "username": {"type": "string", "required": true, "min": 3}
    /// });
    /// 
    /// let validator = LinkValidator::new(&schema).unwrap();
    /// ```
    pub fn new(schema: &Value) -> Result<LinkValidator, String> {
        compile(schema)
    }

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

/// 验证结果
#[derive(Debug)]
pub struct ValidationResult {
    /// 验证是否通过
    pub is_valid: bool,
    /// 错误信息（JSON 格式）
    pub errors: Value,
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
fn compile(schema: &Value) -> Result<LinkValidator, String> {
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

/// 判断给定的值是否为 async-validator 规则格式
fn is_async_rules(value: &Value) -> bool {
    // 简单检查是否为 async-validator 规则格式
    // async-validator 规则通常是对象，其中值是规则对象或规则对象数组
    match value {
        Value::Object(obj) => {
            // 检查对象是否具有 JSON Schema 的特征
            if obj.contains_key("type") && obj.get("type").unwrap().is_string() {
                let type_value = obj.get("type").unwrap().as_str().unwrap();
                // JSON Schema 通常具有这些类型值
                match type_value {
                    "object" | "array" | "string" | "number" | "integer" | "boolean" => {
                        // 进一步检查是否具有 JSON Schema 特征字段
                        if obj.contains_key("properties") || obj.contains_key("items") {
                            return false; // 很可能是 JSON Schema
                        }
                    }
                    _ => ()
                }
            }
            
            // 检查是否具有 JSON Schema 的顶层字段
            if obj.contains_key("properties") || 
               obj.contains_key("items") || 
               obj.contains_key("definitions") ||
               obj.contains_key("additionalProperties") ||
               obj.contains_key("patternProperties") {
                return false; // 明确是 JSON Schema
            }
            
            // 检查字段是否符合 async-validator 规则格式
            for (_, v) in obj {
                match v {
                    // async-validator 规则可以是单个对象
                    Value::Object(rule_obj) => {
                        // 检查对象是否符合 async-validator 规则特征
                        if is_async_rule_object(rule_obj) {
                            return true;
                        }
                    },
                    // async-validator 规则可以是对象数组
                    Value::Array(arr) => {
                        if !arr.is_empty() {
                            if let Value::Object(rule_obj) = &arr[0] {
                                // 检查数组中的对象是否符合 async-validator 规则特征
                                if is_async_rule_object(rule_obj) {
                                    return true;
                                }
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

/// 判断给定的对象是否为 async-validator 规则对象
fn is_async_rule_object(obj: &Map<String, Value>) -> bool {
    // 检查是否包含 async-validator 特有的规则字段
    let async_validator_fields = [
        "type", "required", "min", "max", "len", "pattern", 
        "enum", "whitespace", "fields", "message"
    ];
    
    // 检查是否包含 JSON Schema 特有的字段（这些在 async-validator 中不常见）
    let json_schema_fields = [
        "properties", "items", "additionalProperties", 
        "patternProperties", "definitions", "minProperties",
        "maxProperties", "minItems", "maxItems", "uniqueItems",
        "minLength", "maxLength", "multipleOf", "exclusiveMinimum",
        "exclusiveMaximum", "format"
    ];
    
    // 如果包含 JSON Schema 特有字段，则不是 async-validator 规则
    for field in &json_schema_fields {
        if obj.contains_key(*field) {
            return false;
        }
    }
    
    // 如果包含 async-validator 特有字段，则很可能是 async-validator 规则
    for field in &async_validator_fields {
        if obj.contains_key(*field) {
            return true;
        }
    }
    
    // 默认情况下，如果对象不为空且不包含 JSON Schema 特征，则认为是 async-validator 规则
    !obj.is_empty()
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
            
            if rule.extra.contains_key("transform") {
                unsupported.push(format!("Field '{}': transform option not supported", field_name));
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
        if !field_schema.contains_key("type") && field_rules.iter().any(|r| r.field_type.is_some()) {
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
            fields: None,
            extra: Map::new(),
        }
    }
}

