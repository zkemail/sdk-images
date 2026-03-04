use anyhow::Result;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use relayer_utils::LOG;
use sdk_utils::proto_types::proto_blueprint::Blueprint;
use serde::Deserialize;
use slog::info;

// Import from the crate root
use crate::circuit_generator::generate_circuit;
use crate::filesystem::{
    FileUploader, ProductionFileUploader, UploadTarget, compile_circuit, scaffold_contracts_for_circuit,
    setup, setup_circuit_dir, zip_circuit_dir, zip_regex_graphs,
};
use crate::models::CircuitTemplateInputs;
use crate::regex_generator::generate_regex_circuits;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadUrls {
    pub circuit_1024: String,
    pub circuit_2048: String,
    pub circuit_json_1024: String,
    pub circuit_json_2048: String,
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
    match process_circuits(payload, ProductionFileUploader).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            println!("e while compiling: {:?}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

async fn process_circuits(payload: Payload, uploader: impl FileUploader) -> Result<()> {
    // Setup filesystem
    let tmp_dir = std::path::Path::new("./tmp");
    setup(tmp_dir).await?;

    // Extract blueprint
    let blueprint = payload.blueprint;

    // Generate regex circuits (shared between both key sizes) under this tmp dir
    let regex_graphs_dir = generate_regex_circuits(tmp_dir, &blueprint.decomposed_regexes)?;

    // Generate separate circuits for 1024-bit and 2048-bit RSA keys.
    //
    // Why two circuits instead of one with conditional logic?
    // - Noir circuits are compile-time fixed; any conditional branching on key size
    //   would still compile all code paths and incur the constraint cost of both sizes
    // - Two specialized circuits are more efficient since each only contains the
    //   constraints needed for its specific key size
    // - The zkemail library exports different array sizes (KEY_LIMBS_1024=9 vs
    //   KEY_LIMBS_2048=18) that must be known at compile time for type safety
    // - This approach lets provers select the appropriate circuit based on the
    //   actual DKIM key size of the email they're proving

    // Generate and compile 1024-bit and 2048-bit circuits, capturing their circuit directories.
    let circuit_1024_dir = process_circuit(&blueprint, 1024, tmp_dir, &regex_graphs_dir).await?;
    let circuit_2048_dir = process_circuit(&blueprint, 2048, tmp_dir, &regex_graphs_dir).await?;

    // Derive bytecode JSON paths from the circuit directories
    let bytecode_1024 = circuit_1024_dir.join("target").join("sdk_noir.json");
    let bytecode_2048 = circuit_2048_dir.join("target").join("sdk_noir.json");

    // Zip each circuit and capture the resulting zip paths
    let circuit_1024_zip = zip_circuit_dir(&circuit_1024_dir, "circuit_1024.zip").await?;
    let circuit_2048_zip = zip_circuit_dir(&circuit_2048_dir, "circuit_2048.zip").await?;

    // Zip regex graphs (from their holder dir) and capture the zip path
    let regex_graphs_zip = zip_regex_graphs(&regex_graphs_dir, "regex_graphs.zip").await?;

    let to_string = |p: &std::path::Path| p.to_string_lossy().into_owned();

    let upload_targets = vec![
        UploadTarget {
            url: payload.upload_urls.circuit_1024,
            path: to_string(&circuit_1024_zip),
            content_type: "application/zip".to_string(),
        },
        UploadTarget {
            url: payload.upload_urls.circuit_2048,
            path: to_string(&circuit_2048_zip),
            content_type: "application/zip".to_string(),
        },
        UploadTarget {
            url: payload.upload_urls.circuit_json_1024,
            path: to_string(&bytecode_1024),
            content_type: "application/json".to_string(),
        },
        UploadTarget {
            url: payload.upload_urls.circuit_json_2048,
            path: to_string(&bytecode_2048),
            content_type: "application/json".to_string(),
        },
        UploadTarget {
            url: payload.upload_urls.regex_graphs,
            path: to_string(&regex_graphs_zip),
            content_type: "application/zip".to_string(),
        },
    ];

    uploader.upload_files(upload_targets).await?;

    Ok(())
}

/// Generates a Noir circuit for the given key size, prepares its circuit-specific
/// directory under `tmp_dir`, copies in the shared regex Noir modules from
/// `regex_graphs_dir`, writes `main.nr`, and compiles it with `nargo`.
/// Returns the Noir project root directory path (`<tmp_dir>/<key_size_bits>/noir`).
async fn process_circuit(
    blueprint: &Blueprint,
    key_size_bits: u32,
    tmp_dir: &std::path::Path,
    regex_graphs_dir: &std::path::Path,
) -> Result<std::path::PathBuf> {
    info!(LOG, "Generating {}-bit circuit", key_size_bits);

    // Ensure the circuit-specific tmp directory exists and has `noir/src` + `noir/Nargo.toml`
    let subdir = key_size_bits.to_string();
    setup_circuit_dir(tmp_dir, &subdir).await?;

    let circuit_dir = tmp_dir.join(&subdir).join("noir");

    // Copy shared regex Noir modules into this circuit's src dir
    if regex_graphs_dir.exists() {
        let src_dir = circuit_dir.join("src");
        for entry in std::fs::read_dir(regex_graphs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "nr" {
                    let file_name = entry.file_name();
                    let dest = src_dir.join(file_name);
                    std::fs::copy(&path, dest)?;
                }
            }
        }
    }

    let inputs = CircuitTemplateInputs::from_blueprint_with_key_size(blueprint, key_size_bits);
    let circuit = generate_circuit(inputs)?;

    let main_path = circuit_dir.join("src").join("main.nr");
    std::fs::write(&main_path, &circuit)?;

    compile_circuit(&circuit_dir).await?;

    // After successful compilation (and Honk verifier generation), scaffold the
    // Foundry contracts package for this circuit.
    scaffold_contracts_for_circuit(&circuit_dir, blueprint)?;

    Ok(circuit_dir)
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
                is_hashed: Some(false),
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
            circuit_1024: "".to_string(),
            circuit_2048: "".to_string(),
            circuit_json_1024: "".to_string(),
            circuit_json_2048: "".to_string(),
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
        let result = process_circuits(payload, mock_uploader).await;

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }

        // Assert the result
        assert!(result.is_ok());

        // Verify HonkVerifier.sol is generated for both key sizes
        let honk_1024 = std::path::Path::new("./tmp/1024/target/HonkVerifier.sol");
        assert!(
            honk_1024.exists(),
            "HonkVerifier.sol should be generated for 1024-bit circuit"
        );

        let honk_2048 = std::path::Path::new("./tmp/2048/target/HonkVerifier.sol");
        assert!(
            honk_2048.exists(),
            "HonkVerifier.sol should be generated for 2048-bit circuit"
        );

        // Verify contracts scaffolding exists for both key sizes.
        for key_dir in ["1024", "2048"] {
            let base = std::path::Path::new("./tmp").join(key_dir).join("contracts");

            // Top-level config files
            for name in [
                ".env.example",
                "README.md",
                "foundry.toml",
                "package.json",
                "remappings.txt",
                "yarn.lock",
            ] {
                let path = base.join(name);
                assert!(
                    path.exists(),
                    "Expected contracts config file to exist: {}",
                    path.display()
                );
            }

            // Interfaces
            for name in [
                "IDKIMRegistry.sol",
                "IHonkVerifier.sol",
                "IZKEmailVerifier.sol",
            ] {
                let path = base.join("src").join("interfaces").join(name);
                assert!(
                    path.exists(),
                    "Expected contracts interface file to exist: {}",
                    path.display()
                );
            }

            // Deploy script
            let deploy_script =
                base.join("script").join("DeployZKEmailVerifier.s.sol");
            assert!(
                deploy_script.exists(),
                "Expected deploy script to exist: {}",
                deploy_script.display()
            );

            // HonkVerifier copied under contracts/src
            let honk_under_contracts = base.join("src").join("HonkVerifier.sol");
            assert!(
                honk_under_contracts.exists(),
                "Expected HonkVerifier under contracts/src: {}",
                honk_under_contracts.display()
            );

            // ZKEmailVerifier rendered
            let zkemail = base.join("src").join("ZKEmailVerifier.sol");
            assert!(
                zkemail.exists(),
                "Expected ZKEmailVerifier to exist: {}",
                zkemail.display()
            );

            let zkemail_code = std::fs::read_to_string(&zkemail)
                .unwrap_or_else(|_| panic!("Failed to read {}", zkemail.display()));

            assert!(
                zkemail_code.contains("contract ZKEmailVerifier"),
                "ZKEmailVerifier should define the contract in {}",
                zkemail.display()
            );

            assert!(
                zkemail_code.contains("IHonkVerifier"),
                "ZKEmailVerifier should reference IHonkVerifier in {}",
                zkemail.display()
            );
        }

        // Additionally, ensure the sender domain from the blueprint is wired
        // into at least the 1024-bit ZKEmailVerifier.
        let zkemail_1024 =
            std::path::Path::new("./tmp/1024/contracts/src/ZKEmailVerifier.sol");
        let zkemail_1024_code = std::fs::read_to_string(zkemail_1024)
            .expect("ZKEmailVerifier for 1024-bit circuit must exist");
        assert!(
            zkemail_1024_code.contains("x.com"),
            "ZKEmailVerifier for 1024-bit circuit should embed the sender domain 'x.com'"
        );
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
            circuit_name: "AppleKYC".to_string(),
            ignore_body_hash_check: false, // Set to false to enable body masking test
            sha_precompute_selector: "".to_string(),
            email_body_max_length: 2048, // Set a valid body length for body masking
            sender_domain: "email.apple.com".to_string(),
            enable_header_masking: true, // Enable header masking for testing
            enable_body_masking: true,   // Enable body masking for testing
            client_zk_framework: 3,      // Noir
            server_zk_framework: 0,      // None
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
                    is_hashed: Some(false),
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
            circuit_1024: "".to_string(),
            circuit_2048: "".to_string(),
            circuit_json_1024: "".to_string(),
            circuit_json_2048: "".to_string(),
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
        let result = process_circuits(payload, mock_uploader).await;

        println!("Got a result");

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }

        // Assert the result
        assert!(result.is_ok());

        // Verify body_mask is generated as a function input parameter in the 1024-bit circuit
        let circuit_path = std::path::Path::new("./tmp/1024/noir/src/main.nr");
        let circuit_code = std::fs::read_to_string(circuit_path)
            .expect("Generated 1024-bit circuit main.nr must exist");

        // Verify header_mask is a function input parameter
        assert!(
            circuit_code.contains("header_mask: [bool;"),
            "Generated circuit should have 'header_mask' as a function input parameter when enable_header_masking is true"
        );

        // Verify body_mask is a function input parameter
        assert!(
            circuit_code.contains("body_mask: [bool;"),
            "Generated circuit should have 'body_mask' as a function input parameter when enable_body_masking is true"
        );

        // Verify masked outputs are present
        assert!(
            circuit_code.contains("masked_header"),
            "Generated circuit should contain 'masked_header' output"
        );
        assert!(
            circuit_code.contains("masked_body"),
            "Generated circuit should contain 'masked_body' output"
        );

        println!(
            "✓ Body mask verified in test_compile_circuit_apple - body_mask is generated as a function input parameter"
        );
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
                is_hashed: Some(false),
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
            circuit_1024: "".to_string(),
            circuit_2048: "".to_string(),
            circuit_json_1024: "".to_string(),
            circuit_json_2048: "".to_string(),
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
        let result = process_circuits(payload, mock_uploader).await;

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
                is_hashed: Some(true),
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
            circuit_1024: "".to_string(),
            circuit_2048: "".to_string(),
            circuit_json_1024: "".to_string(),
            circuit_json_2048: "".to_string(),
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
        let result = process_circuits(payload, mock_uploader).await;

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
                is_hashed: Some(false),
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
            circuit_1024: "".to_string(),
            circuit_2048: "".to_string(),
            circuit_json_1024: "".to_string(),
            circuit_json_2048: "".to_string(),
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
        let result = process_circuits(payload, mock_uploader).await;

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }

        // Assert the result
        assert!(result.is_ok());
    }
}
