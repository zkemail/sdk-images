use std::fmt;

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};
use uuid::Uuid;

// Enums and Structs

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ZkFramework {
    Circom,
    // Add other variants as needed
}

#[derive(Serialize, Debug, Clone)]
pub enum Status {
    Draft,
    InProgress,
    Done,
    Failed,
    // Add other variants as needed
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StatusVisitor;

        impl<'de> Visitor<'de> for StatusVisitor {
            type Value = Status;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an integer between 1 and 4")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Status, E>
            where
                E: de::Error,
            {
                match value {
                    1 => Ok(Status::Draft),
                    2 => Ok(Status::InProgress),
                    3 => Ok(Status::Done),
                    4 => Ok(Status::Failed),
                    _ => Err(de::Error::invalid_value(
                        de::Unexpected::Unsigned(value),
                        &self,
                    )),
                }
            }
        }

        deserializer.deserialize_u64(StatusVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalInput {
    pub name: String,
    pub max_length: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DecomposedRegexPart {
    pub is_public: bool,
    pub regex_def: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DecomposedRegex {
    pub parts: Vec<DecomposedRegexPart>,
    pub name: String,
    pub max_length: usize,
    pub location: String,
    pub is_hashed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Timestamp {
    seconds: usize,
    nanos: usize,
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
    pub email_header_max_length: Option<usize>,
    pub email_body_max_length: Option<usize>,
    pub sender_domain: Option<String>,
    pub enable_header_masking: Option<bool>,
    pub enable_body_masking: Option<bool>,
    pub zk_framework: Option<ZkFramework>,
    pub is_public: Option<bool>,
    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
    pub external_inputs: Option<Vec<ExternalInput>>,
    pub decomposed_regexes: Option<Vec<DecomposedRegex>>,
    pub status: Option<Status>,
    pub verifier_contract_chain: Option<usize>,
    pub verifier_contract_address: Option<String>,
    pub version: Option<usize>,
}
