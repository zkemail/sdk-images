use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Enums and Structs

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone)]
#[sqlx(type_name = "zk_framework_enum", rename_all = "lowercase")]
pub enum ZkFramework {
    Circom,
    // Add other variants as needed
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone)]
#[sqlx(type_name = "status_enum", rename_all = "lowercase")]
pub enum Status {
    Draft,
    InProgress,
    Done,
    Failed,
    // Add other variants as needed
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalInput {
    pub name: String,
    #[serde(rename = "maxLength")]
    pub max_length: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DecomposedRegexPart {
    #[serde(rename = "isPublic")]
    pub is_public: bool,
    #[serde(rename = "regexDef")]
    pub regex_def: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DecomposedRegex {
    pub parts: Vec<DecomposedRegexPart>,
    pub name: String,
    #[serde(rename = "maxLength")]
    pub max_length: i32,
    pub location: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blueprint {
    pub id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub slug: String,
    pub tags: Option<Vec<String>>,
    pub github_username: Option<String>,
    pub email_query: Option<String>,
    pub circuit_name: Option<String>,
    pub ignore_body_hash_check: Option<bool>,
    pub remove_soft_line_breaks: Option<bool>,
    pub sha_precompute_selector: Option<String>,
    pub email_header_max_length: Option<i32>,
    pub email_body_max_length: Option<i32>,
    pub sender_domain: Option<String>,
    pub enable_header_masking: Option<bool>,
    pub enable_body_masking: Option<bool>,
    pub zk_framework: Option<ZkFramework>,
    pub is_public: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub external_inputs: Option<Vec<ExternalInput>>,
    pub decomposed_regexes: Vec<DecomposedRegex>,
    pub status: Option<Status>,
    pub verifier_contract_chain: Option<i32>,
    pub verifier_contract_address: Option<String>,
    pub version: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payload {
    pub blueprint: Blueprint,
    #[serde(rename = "uploadUrl")]
    pub upload_url: String,
}
