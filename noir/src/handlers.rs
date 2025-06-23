use anyhow::Result;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use relayer_utils::LOG;
use sdk_utils::proto_types::proto_blueprint::Blueprint;
use serde::Deserialize;
use slog::info;

// Import from the crate root
use crate::circuit_generator::generate_circuit;
use crate::filesystem::{FileUploader, ProductionFileUploader, cleanup, compile_circuit, setup};
use crate::models::CircuitTemplateInputs;
use crate::regex_generator::generate_regex_circuits;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadUrls {
    pub circuit: String,
    pub circuit_json: String,
    pub regex_graphs: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub blueprint: Blueprint,
    pub upload_urls: UploadUrls,
    pub database_url: String,
    pub private_key: String,
    pub rpc_url: String,
    pub chain_id: u32,
    pub etherscan_api_key: String,
    pub dkim_registry_address: String,
}

pub async fn compile_circuit_handler(
    Json(payload): Json<Payload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!(LOG, "Received payload: {:?}", payload);

    // Process the request
    match process_circuit(payload, ProductionFileUploader).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            println!("e while compiling: {:?}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

async fn process_circuit(payload: Payload, uploader: impl FileUploader) -> Result<()> {
    // Setup filesystem
    setup().await?;

    // Extract blueprint
    let blueprint = payload.blueprint;

    // Generate regex circuits
    generate_regex_circuits(&blueprint.decomposed_regexes)?;

    // Generate main circuit from template
    let circuit_template_inputs = CircuitTemplateInputs::from(blueprint);

    let circuit = generate_circuit(circuit_template_inputs)?;

    // Write the circuit to a file
    let circuit_path = "./tmp/src/main.nr";
    std::fs::write(circuit_path, circuit)?;

    // Compile and clean up
    compile_circuit().await?;

    cleanup().await?;

    // Upload files
    uploader.upload_files(payload.upload_urls).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::MockFileUploader;
    // use dotenv::dotenv;
    use prost_wkt_types::Timestamp;
    use sdk_utils::proto_types::proto_blueprint::{
        Blueprint, DecomposedRegex, DecomposedRegexPart, ExternalInput,
    };
    // use std::env;

    #[tokio::test]
    async fn test_compile_circuit_x_export_data() {
        let mut mock_uploader = MockFileUploader::new();
        mock_uploader
            .expect_upload_files()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let blueprint = Blueprint {
            internal_version: "v2".to_string(),
            id: "4478f3bc-9ba8-4906-ba87-09fc049cef46".to_string(),
            title: "XAccountExportData".to_string(),
            description:
                "Prove you've asked to export your twitter/X data and reveal only the download link"
                    .to_string(),
            slug: "DimiDumo/XAccountExportData".to_string(),
            tags: vec![],
            email_query: "from:x.com".to_string(),
            use_new_sdk: false,
            circuit_name: "XAccountExportData".to_string(),
            ignore_body_hash_check: false,
            sha_precompute_selector: "".to_string(),
            email_body_max_length: 6208,
            sender_domain: "x.com".to_string(),
            enable_header_masking: false,
            enable_body_masking: false,
            client_zk_framework: 3, // Noir
            server_zk_framework: 0, // None
            verifier_contract_chain: 84532,
            verifier_contract_address: "0x6679b65c5CFCba507Bf105491A3b5B68764B1464".to_string(),
            is_public: true,
            created_at: Some(Timestamp {
                seconds: 1746574183,
                nanos: 310124000,
            }),
            updated_at: Some(Timestamp {
                seconds: 1746574183,
                nanos: 310124000,
            }),
            external_inputs: vec![],
            decomposed_regexes: vec![DecomposedRegex {
                name: "download_data_link".to_string(),
                max_match_length: 128,
                location: "body".to_string(),
                is_hashed: false,
                parts: vec![
                    DecomposedRegexPart {
                        is_public: Some(false),
                        regex_def: "ready for you to download ".to_string(),
                        max_length: None,
                    },
                    DecomposedRegexPart {
                        is_public: Some(true),
                        regex_def: "[^ ]*".to_string(),
                        max_length: Some(20),
                    },
                ],
            }],
            client_status: 1, // InProgress
            server_status: 3, // Done
            version: 1,
            github_username: "DimiDumo".to_string(),
            email_header_max_length: 1024,
            remove_soft_linebreaks: true,
            stars: 0,
            ptau: 0,
            num_local_proofs: 0,
        };

        let upload_urls = UploadUrls {
            circuit: "".to_string(),
            circuit_json: "".to_string(),
            regex_graphs: "".to_string(),
        };

        let payload = Payload {
            blueprint,
            upload_urls,
            database_url: "".to_string(),
            private_key: "".to_string(),
            rpc_url: "".to_string(),
            chain_id: 84532,
            etherscan_api_key: "".to_string(),
            dkim_registry_address: "".to_string(),
        };

        // Call the handler with the mock uploader
        let result = process_circuit(payload, mock_uploader).await;

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }

        // Assert the result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compile_circuit_apple() {
        // Set up test environment
        // dotenv().ok();
        // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let database_url = "".to_string();

        // Set up mock uploader
        let mut mock_uploader = MockFileUploader::new();
        mock_uploader
            .expect_upload_files()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let blueprint = Blueprint {
            internal_version: "v2".to_string(),
            id: "88802381-0501-4c4a-bcb5-03fdeacf453e".to_string(),
            title: "AppleKYC".to_string(),
            description: "Prove you have a valid Apple account".to_string(),
            slug: "DimiDumo/AppleKYC".to_string(),
            tags: vec![],
            email_query: "from:email.apple.com".to_string(),
            use_new_sdk: false,
            circuit_name: "AppleKYC".to_string(),
            ignore_body_hash_check: true,
            sha_precompute_selector: "".to_string(),
            email_body_max_length: 0,
            sender_domain: "email.apple.com".to_string(),
            enable_header_masking: false,
            enable_body_masking: false,
            client_zk_framework: 3, // Noir
            server_zk_framework: 0, // None
            verifier_contract_chain: 84532,
            verifier_contract_address: "0x1E8AbE8B8551E73d25239004EffccA2d077eF146".to_string(),
            is_public: true,
            created_at: Some(Timestamp {
                seconds: 1746538605,
                nanos: 86528000,
            }),
            updated_at: Some(Timestamp {
                seconds: 1746538605,
                nanos: 86528000,
            }),
            external_inputs: vec![ExternalInput {
                name: "address".to_string(),
                max_length: 44,
            }],
            decomposed_regexes: vec![
                DecomposedRegex {
                    name: "subject".to_string(),
                    max_match_length: 256,
                    location: "header".to_string(),
                    is_hashed: false,
                    parts: vec![
                        DecomposedRegexPart {
                            is_public: Some(false),
                            regex_def: "(?:\r\n|^)subject:".to_string(),
                            max_length: None,
                        },
                        DecomposedRegexPart {
                            is_public: Some(true),
                            regex_def: "[^\r\n]+".to_string(),
                            max_length: Some(20),
                        },
                        DecomposedRegexPart {
                            is_public: Some(false),
                            regex_def: "\r\n".to_string(),
                            max_length: None,
                        },
                    ],
                }, // Other DecomposedRegex objects omitted for brevity - add them if needed
            ],
            client_status: 1, // InProgress
            server_status: 3, // Done
            version: 6,
            github_username: "DimiDumo".to_string(),
            email_header_max_length: 2048,
            remove_soft_linebreaks: true,
            stars: 0,
            ptau: 0,
            num_local_proofs: 0,
        };

        let upload_urls = UploadUrls {
            circuit: "".to_string(),
            circuit_json: "".to_string(),
            regex_graphs: "".to_string(),
        };

        let payload = Payload {
            blueprint,
            upload_urls,
            database_url,
            private_key: "".to_string(),
            rpc_url: "".to_string(),
            chain_id: 84532,
            etherscan_api_key: "".to_string(),
            dkim_registry_address: "".to_string(),
        };

        println!("calling process_circuit");

        // Call the handler with the mock uploader
        let result = process_circuit(payload, mock_uploader).await;

        println!("Got a result");

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }

        // Assert the result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compile_circuit_registry() {
        // Set up mock uploader
        let mut mock_uploader = MockFileUploader::new();
        mock_uploader
            .expect_upload_files()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let blueprint = Blueprint {
            internal_version: "v2".to_string(),
            id: "87ec6e2f-ca5a-4af8-ac85-2e2cc94602f0".to_string(),
            title: "Sp1Residency".to_string(),
            description: "Sp1Residency".to_string(),
            slug: "DimiDumo/sp1_residency".to_string(),
            tags: vec![],
            email_query: "".to_string(),
            use_new_sdk: false,
            circuit_name: "sp1_residency".to_string(),
            ignore_body_hash_check: true,
            sha_precompute_selector: "".to_string(),
            email_body_max_length: 0,
            sender_domain: "succinct.xyz".to_string(),
            enable_header_masking: false,
            enable_body_masking: false,
            client_zk_framework: 3, // Noir
            server_zk_framework: 0, // None
            verifier_contract_chain: 0,
            verifier_contract_address: "".to_string(),
            is_public: true,
            created_at: Some(Timestamp {
                seconds: 1746543161,
                nanos: 36149000,
            }),
            updated_at: Some(Timestamp {
                seconds: 1746543161,
                nanos: 36149000,
            }),
            external_inputs: vec![],
            decomposed_regexes: vec![DecomposedRegex {
                name: "subject".to_string(),
                max_match_length: 50,
                location: "header".to_string(),
                is_hashed: false,
                parts: vec![
                    DecomposedRegexPart {
                        is_public: Some(false),
                        regex_def: "Welcome ".to_string(),
                        max_length: None,
                    },
                    DecomposedRegexPart {
                        is_public: Some(true),
                        regex_def: "to the ".to_string(),
                        max_length: Some(20),
                    },
                    DecomposedRegexPart {
                        is_public: Some(false),
                        regex_def: "Succinct ZK Residency!".to_string(),
                        max_length: None,
                    },
                ],
            }],
            client_status: 1, // InProgress
            server_status: 3, // Done
            version: 31,
            github_username: "DimiDumo".to_string(),
            email_header_max_length: 896,
            remove_soft_linebreaks: false,
            stars: 0,
            ptau: 0,
            num_local_proofs: 0,
        };

        let upload_urls = UploadUrls {
            circuit: "".to_string(),
            circuit_json: "".to_string(),
            regex_graphs: "".to_string(),
        };

        let payload = Payload {
            blueprint,
            upload_urls,
            database_url: "".to_string(),
            private_key: "".to_string(),
            rpc_url: "".to_string(),
            chain_id: 0,
            etherscan_api_key: "".to_string(),
            dkim_registry_address: "".to_string(),
        };

        // Call the handler with the mock uploader
        let result = process_circuit(payload, mock_uploader).await;

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }

        // Assert the result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compile_circuit_kraken() {
        // Set up mock uploader
        let mut mock_uploader = MockFileUploader::new();
        mock_uploader
            .expect_upload_files()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let blueprint = Blueprint {
            internal_version: "v2".to_string(),
            id: "85255ee2-acfe-49ca-959c-edd009b53bb5".to_string(),
            title: "Kraken KYC (Intermediate)".to_string(),
            description: "Proof of Kraken Intermediate Account".to_string(),
            slug: "Bisht13/krakenintermediate".to_string(),
            tags: vec![],
            email_query: "from:kraken.com".to_string(),
            use_new_sdk: false,
            circuit_name: "krakenintermediate".to_string(),
            ignore_body_hash_check: true,
            sha_precompute_selector: "".to_string(),
            email_body_max_length: 4096,
            sender_domain: "kraken.com".to_string(),
            enable_header_masking: false,
            enable_body_masking: false,
            client_zk_framework: 3, // Noir
            server_zk_framework: 0, // None
            verifier_contract_chain: 84532,
            verifier_contract_address: "".to_string(),
            is_public: true,
            created_at: Some(Timestamp {
                seconds: 1736325873,
                nanos: 967251000,
            }),
            updated_at: Some(Timestamp {
                seconds: 1736326473,
                nanos: 627382000,
            }),
            external_inputs: vec![ExternalInput {
                name: "test".to_string(),
                max_length: 4096,
            }],
            decomposed_regexes: vec![DecomposedRegex {
                name: "email_subject".to_string(),
                max_match_length: 64,
                location: "header".to_string(),
                is_hashed: true,
                parts: vec![
                    DecomposedRegexPart {
                        is_public: Some(true),
                        regex_def: "subject:".to_string(),
                        max_length: Some(20),
                    },
                    DecomposedRegexPart {
                        is_public: Some(true),
                        regex_def: "Good news: your account is now Intermediate!".to_string(),
                        max_length: Some(20),
                    },
                ],
            }],
            client_status: 1, // InProgress
            server_status: 3, // Done
            version: 1,
            github_username: "Bisht13".to_string(),
            email_header_max_length: 1088,
            remove_soft_linebreaks: false,
            stars: 0,
            ptau: 0,
            num_local_proofs: 0,
        };

        let upload_urls = UploadUrls {
            circuit: "".to_string(),
            circuit_json: "".to_string(),
            regex_graphs: "".to_string(),
        };

        let payload = Payload {
            blueprint,
            upload_urls,
            database_url: "".to_string(),
            private_key: "".to_string(),
            rpc_url: "https://sepolia.base.org".to_string(),
            chain_id: 84532,
            etherscan_api_key: "".to_string(),
            dkim_registry_address: "0x2971369F8681aF91F434D6F0f599C07842F3A17e".to_string(),
        };

        // Call the handler with the mock uploader
        let result = process_circuit(payload, mock_uploader).await;

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }

        // Assert the result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compile_circuit_subject_extract() {
        // Set up mock uploader
        let mut mock_uploader = MockFileUploader::new();
        mock_uploader
            .expect_upload_files()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let blueprint = Blueprint {
            internal_version: "v2".to_string(),
            id: "87ec6e2f-ca5a-4af8-ac85-2e2cc94602f0".to_string(),
            title: "Sp1Residency".to_string(),
            description: "Sp1Residency".to_string(),
            slug: "DimiDumo/sp1_residency".to_string(),
            tags: vec![],
            email_query: "".to_string(),
            use_new_sdk: false,
            circuit_name: "sp1_residency".to_string(),
            ignore_body_hash_check: true,
            sha_precompute_selector: "".to_string(),
            email_body_max_length: 0,
            sender_domain: "succinct.xyz".to_string(),
            enable_header_masking: false,
            enable_body_masking: false,
            client_zk_framework: 3, // Noir
            server_zk_framework: 0, // None
            verifier_contract_chain: 0,
            verifier_contract_address: "".to_string(),
            is_public: true,
            created_at: Some(Timestamp {
                seconds: 1746543161,
                nanos: 36149000,
            }),
            updated_at: Some(Timestamp {
                seconds: 1746543161,
                nanos: 36149000,
            }),
            external_inputs: vec![],
            decomposed_regexes: vec![DecomposedRegex {
                name: "subject".to_string(),
                max_match_length: 64,
                location: "header".to_string(),
                is_hashed: false,
                parts: vec![
                    DecomposedRegexPart {
                        is_public: Some(false),
                        regex_def: "(?:\r\n|^)subject:".to_string(),
                        max_length: None,
                    },
                    DecomposedRegexPart {
                        is_public: Some(true),
                        regex_def: "[a-z]+".to_string(),
                        max_length: Some(20),
                    },
                    DecomposedRegexPart {
                        is_public: Some(false),
                        regex_def: "\r\n".to_string(),
                        max_length: None,
                    },
                ],
            }],
            client_status: 1, // InProgress
            server_status: 3, // Done
            version: 31,
            github_username: "DimiDumo".to_string(),
            email_header_max_length: 896,
            remove_soft_linebreaks: false,
            stars: 0,
            ptau: 0,
            num_local_proofs: 0,
        };

        let upload_urls = UploadUrls {
            circuit: "".to_string(),
            circuit_json: "".to_string(),
            regex_graphs: "".to_string(),
        };

        let payload = Payload {
            blueprint,
            upload_urls,
            database_url: "".to_string(),
            private_key: "".to_string(),
            rpc_url: "".to_string(),
            chain_id: 0,
            etherscan_api_key: "".to_string(),
            dkim_registry_address: "".to_string(),
        };

        // Call the handler with the mock uploader
        let result = process_circuit(payload, mock_uploader).await;

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }

        // Assert the result
        assert!(result.is_ok());
    }
}
