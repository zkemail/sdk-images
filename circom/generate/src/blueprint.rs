use anyhow::{Error, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, Pool, Postgres};
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

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone)]
#[sqlx(type_name = "verifier_contract")]
pub struct VerifierContract {
    pub chain: i32,
    pub address: String,
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
    pub verifier_contract: Option<VerifierContract>,
    pub version: Option<i32>,
}

// Intermediate struct to match database types
#[derive(Debug)]
struct BlueprintRaw {
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
    pub external_inputs: Option<Json<Vec<ExternalInput>>>,
    pub decomposed_regexes: Json<Vec<DecomposedRegex>>,
    pub status: Option<Status>,
    pub verifier_contract: Option<VerifierContract>,
    pub version: Option<i32>,
}

// Implement TryFrom to convert BlueprintRaw to Blueprint
impl TryFrom<BlueprintRaw> for Blueprint {
    type Error = Error;

    fn try_from(raw: BlueprintRaw) -> Result<Self> {
        Ok(Blueprint {
            id: raw.id,
            title: raw.title,
            description: raw.description,
            slug: raw.slug,
            tags: raw.tags,
            github_username: raw.github_username,
            email_query: raw.email_query,
            circuit_name: raw.circuit_name,
            ignore_body_hash_check: raw.ignore_body_hash_check,
            remove_soft_line_breaks: raw.remove_soft_line_breaks,
            sha_precompute_selector: raw.sha_precompute_selector,
            email_header_max_length: raw.email_header_max_length,
            email_body_max_length: raw.email_body_max_length,
            sender_domain: raw.sender_domain,
            enable_header_masking: raw.enable_header_masking,
            enable_body_masking: raw.enable_body_masking,
            zk_framework: raw.zk_framework,
            is_public: raw.is_public,
            created_at: raw.created_at,
            updated_at: raw.updated_at,
            external_inputs: raw.external_inputs.map(|json| json.0),
            decomposed_regexes: raw.decomposed_regexes.0,
            status: raw.status,
            verifier_contract: raw.verifier_contract,
            version: raw.version,
        })
    }
}

pub async fn get_blueprint(pool: &Pool<Postgres>, id: Uuid) -> Result<Blueprint> {
    let raw_blueprint = sqlx::query_as!(
        BlueprintRaw,
        r#"
        SELECT
            id,
            title,
            description,
            slug,
            tags,
            github_username,
            email_query,
            circuit_name,
            ignore_body_hash_check,
            remove_soft_line_breaks,
            sha_precompute_selector,
            email_header_max_length,
            email_body_max_length,
            sender_domain,
            enable_header_masking,
            enable_body_masking,
            zk_framework as "zk_framework: ZkFramework",
            is_public,
            created_at as "created_at: NaiveDateTime",
            updated_at as "updated_at: NaiveDateTime",
            external_inputs as "external_inputs: Json<Vec<ExternalInput>>",
            decomposed_regexes as "decomposed_regexes: Json<Vec<DecomposedRegex>>",
            status as "status: Status",
            verifier_contract as "verifier_contract: VerifierContract",
            version
        FROM blueprints WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;

    let blueprint = Blueprint::try_from(raw_blueprint)?;

    Ok(blueprint)
}
