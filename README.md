# link-validate

link-validate 是一个将 async-validator 风格验证规则转换为 JSON Schema 并进行验证的 Rust 库。

## 功能概述

1. **自动格式检测**：自动检测输入的 schema 是 JSON Schema 还是 async-validator 规则格式
2. **格式转换**：将 async-validator 规则转换为标准的 JSON Schema
3. **数据验证**：使用 JSON Schema 验证数据
4. **编译检查**：编译 schema 并检查有效性

## 支持的转换规则

### 基础类型转换
- `string` -> JSON Schema string 类型
- `number` -> JSON Schema number 类型
- `integer` -> JSON Schema integer 类型
- `boolean` -> JSON Schema boolean 类型
- `array` -> JSON Schema array 类型
- `object` -> JSON Schema object 类型

### 特殊类型转换
- `method` -> JSON Schema object 类型（标记为 Function 实例）
- `regexp` -> JSON Schema string 类型
- `date` -> JSON Schema string 类型 + date-time format
- `email` -> JSON Schema string 类型 + email format
- `url` -> JSON Schema string 类型 + uri format
- `hex` -> JSON Schema string 类型 + hex pattern
- `any` -> JSON Schema 无类型限制

### 验证规则转换
- `required` -> JSON Schema required 字段
- `min`/`max` -> 根据类型转换为 minLength/maxLength 或 minimum/maximum
- `len` -> 转换为 minLength 和 maxLength (字符串) 或 minItems/maxItems (数组)
- `pattern` -> JSON Schema pattern (正则表达式)
- `enum` -> JSON Schema enum (枚举值)
- `fields` -> JSON Schema properties (嵌套对象)

### 不支持的规则
以下规则不支持转换，会在转换时输出警告：
- `validator` (自定义验证函数)
- `asyncValidator` (异步验证函数)
- `trigger` (触发方式)
- `whitespace` (空白字符处理)
- `transform` (值转换)

## 安装

在你的 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
link-validate = "0.1"
```

## API 文档

### 核心函数

#### `compile`
```rust
pub fn compile(schema: &Value) -> Result<LinkValidator, String>
```

编译 schema 并返回 LinkValidator 验证器。该函数会：

1. 自动检测 schema 格式（JSON Schema 或 async-validator 规则）
2. 如果是 async-validator 规则格式，会自动转换为 JSON Schema
3. 编译 schema
4. 返回 LinkValidator 验证器

#### `LinkValidator::validate`
```rust
impl LinkValidator {
    pub fn validate(&self, data: &Value) -> ValidationResult
}
```

使用 LinkValidator 验证器验证数据。

### 参数说明

- `schema`: 要编译的 schema（JSON 格式），可以是 JSON Schema 或 async-validator 规则格式
- `data`: 要验证的数据（JSON 格式）

### 返回值说明

- `LinkValidator` 验证器，包含编译后的 schema 和原始格式信息
- `ValidationResult` 结构体，包含验证结果和错误信息

## 使用示例

### 基本用法

```rust
use link_validate::{compile, ValidationResult};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = json!({
        "username": {"type": "string", "required": true, "min": 3},
        "email": {"type": "email", "required": true}
    });
    
    // 编译 schema 获取验证器
    let validator = compile(&schema)?;
    
    // 使用验证器多次验证不同数据
    let data1 = json!({"username": "john", "email": "john@example.com"});
    let result1 = validator.validate(&data1);
    
    let data2 = json!({"username": "jane", "email": "jane@example.com"});
    let result2 = validator.validate(&data2);
    
    println!("Result 1: {:?}", result1.is_valid);
    println!("Result 2: {:?}", result2.is_valid);
    
    Ok(())
}
```

### 深度嵌套对象验证

```rust
use link_validate::{compile, ValidationResult};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 支持深度嵌套的对象验证
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
    
    // 编译 schema 获取验证器
    let validator = compile(&schema)?;
    
    // 验证深度嵌套的数据
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
    println!("Validation result: {:?}", result.is_valid);
    
    Ok(())
}
```

## 错误格式说明

根据原始 schema 的类型，返回的错误信息格式会有所不同：

### JSON Schema 错误格式
```json
[
  {
    "message": "Validation error message",
    "instancePath": "/field"
  }
]
```

### async-validator 错误格式
```json
[
  {
    "message": "Validation error message",
    "field": "/field"
  }
]
```

## 支持的 async-validator 规则格式

### 1. 对象格式（单个规则）
```json
{
  "field_name": {
    "type": "string",
    "required": true,
    "min": 3
  }
}
```

### 2. 数组格式（多个规则）
```json
{
  "field_name": [
    {
      "type": "string",
      "required": true
    },
    {
      "min": 3
    }
  ]
}
```

### 3. 嵌套对象格式
```json
{
  "user": {
    "type": "object",
    "required": true,
    "fields": {
      "name": {"type": "string", "required": true},
      "age": {"type": "integer", "minimum": 0}
    }
  }
}
```

## 设计理念

本库的设计遵循以下原则：

1. **简单易用**：提供简洁的公共接口，隐藏内部实现细节
2. **自动检测**：自动检测和处理不同的 schema 格式
3. **格式兼容**：生成符合标准的 JSON Schema，可与其他 JSON Schema 工具配合使用
4. **错误适配**：根据原始 schema 类型返回相应格式的错误信息
5. **明确提示**：对于不支持的规则，明确告知用户并输出警告
6. **性能优化**：提供编译和验证分离的接口，避免重复编译

## 限制

1. 不支持自定义验证函数（validator 和 asyncValidator）
2. 不支持触发方式（trigger）的转换
3. 不支持空白字符处理（whitespace）的转换
4. 不支持值转换（transform）的转换

对于这些不支持的规则，建议在应用层单独处理。

## 贡献

欢迎提交 Issue 和 Pull Request 来改进这个库。

## 许可证

本项目采用 MIT 许可证。