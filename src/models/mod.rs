use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Browser {
    pub id: i64,
    pub name: String,
    pub english_name: String,
    pub exe_path: String,
    pub icon_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParamValueType {
    None,   // 无值，仅标志位
    Text,   // 字符串
    Number, // 数值
}

impl ParamValueType {
    pub fn label(&self) -> &str {
        match self {
            ParamValueType::None => "无",
            ParamValueType::Text => "字符串",
            ParamValueType::Number => "数值",
        }
    }

    pub fn all() -> &'static [ParamValueType] {
        &[ParamValueType::None, ParamValueType::Text, ParamValueType::Number]
    }

    pub fn as_str(&self) -> &str {
        match self {
            ParamValueType::None => "none",
            ParamValueType::Text => "text",
            ParamValueType::Number => "number",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "none" => ParamValueType::None,
            "text" => ParamValueType::Text,
            "number" => ParamValueType::Number,
            _ => ParamValueType::Text,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserParam {
    pub id: i64,
    pub browser_id: i64,
    pub param_name: String,
    pub value_type: ParamValueType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub profile_id: i64,
    pub username: String,
    pub password: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileBrowser {
    pub id: i64,
    pub profile_id: i64,
    pub browser_id: i64,
    pub launch_args: String, // JSON: {"param_id": "value", ...}
    pub created_at: String,
    pub updated_at: String,
}
