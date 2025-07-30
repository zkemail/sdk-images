mod contract;
mod db;
mod payload;
mod template;

use std::{cmp::max, fs, path::Path};

use anyhow::Result;
use contract::{
    create_contract, deploy_verifier_contract, generate_verifier_contract, prepare_contract_data,
};
use db::update_verifier_contract_address;
use payload::UploadUrls;
use rand::Rng;
use relayer_utils::LOG;
use sdk_utils::{
    proto_types::proto_blueprint::Blueprint, run_command, run_command_and_return_output,
    run_command_with_input, upload_to_url,
};
use slog::info;
use sqlx::postgres::PgPoolOptions;
use template::{generate_circuit, generate_regex_circuits, CircuitTemplateInputs};

#[tokio::main]
async fn main() -> Result<()> {
    let payload = payload::load_payload()?;
    info!(LOG, "Loaded configuration: {:?}", payload);
    println!("payload: {:?}", payload);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&payload.database_url)
        .await?;
    println!("Database connection established");

    let blueprint = payload.clone().blueprint;

    let ptau: usize = process_circuit(blueprint.clone()).await?;

    println!("ptau: {}", ptau);

    generate_keys("tmp", ptau).await?;

    let contract_data = prepare_contract_data(&payload);

    create_contract(&contract_data)?;

    // We use two different snarkjs paths:
    // 1. snarkjs_path: The global snarkjs installation for server-side proofs (full zkey)
    // 2. chunked_snarkjs_path: The local node_modules installation for client-side proofs (chunked zkey)
    let snarkjs_path = run_command_and_return_output("which", &["snarkjs"], None)
        .await?
        .trim()
        .to_string();
    let chunked_snarkjs_path = "./node_modules/.bin/snarkjs";

    // Generate verifier contract for client-side proofs using chunked zkey
    generate_verifier_contract(
        "tmp",
        chunked_snarkjs_path,
        "circuit.zkey",
        "ClientProofVerifier",
    )
    .await?;

    // Generate verifier contract for server-side proofs using full zkey
    generate_verifier_contract(
        "tmp",
        &snarkjs_path,
        "circuit_full.zkey",
        "ServerProofVerifier",
    )
    .await?;

    let contract_address = deploy_verifier_contract(payload.clone()).await?;

    info!(LOG, "Contract deployed at: {}", contract_address);

    cleanup().await?;

    update_verifier_contract_address(&pool, &blueprint.id, &contract_address).await?;

    upload_files(payload.upload_urls).await?;

    Ok(())
}

async fn setup() -> Result<()> {
    // Define the tmp path
    let tmp_path = Path::new("./tmp");

    // If tmp exists, remove its contents
    if tmp_path.exists() {
        for entry in fs::read_dir(tmp_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
        }
    } else {
        // If tmp doesn't exist, create it
        fs::create_dir_all(tmp_path)?;
    }

    // Ensure the regex directory exists inside tmp
    let regex_path = tmp_path.join("regex");
    if regex_path.exists() {
        fs::remove_dir_all(&regex_path)?;
    }
    fs::create_dir_all(&regex_path)?;

    run_command("cp", &["package.json", "./tmp"], None).await?;
    Ok(())
}

async fn compile_circuit() -> Result<usize> {
    // Run yarn install in the tmp folder
    info!(LOG, "Running yarn install");
    run_command("yarn", &[], Some("tmp")).await?;

    // Compile the circuit
    info!(LOG, "Compiling circuit");
    let compile_result = run_command_and_return_output(
        "circom",
        &[
            "circuit.circom",
            // "--O2",
            "--sym",
            "--r1cs",
            "--c",
            "--wasm",
            "-l",
            "./node_modules",
        ],
        Some("tmp"),
    )
    .await?;

    // Find the number of non-linear constraints to determine the power of tau
    // Initialize variables for constraints and wires
    let mut constraints = 0u64;
    let mut wires = 0u64;

    // Iterate over each line in the input string
    for line in compile_result.lines() {
        if line.starts_with("non-linear constraints:") {
            // Extract the number of constraints
            if let Some(value_str) = line.split(':').nth(1) {
                constraints = value_str.trim().parse::<u64>().unwrap_or(0);
            }
        } else if line.starts_with("wires:") {
            // Extract the number of wires
            if let Some(value_str) = line.split(':').nth(1) {
                wires = value_str.trim().parse::<u64>().unwrap_or(0);
            }
        }
    }

    // Determine the maximum value between constraints and wires
    let max_value = max(constraints, wires);

    // Compute the minimal power k such that 2^k > max_value
    let mut k = 0u32;
    while 1u64 << k <= max_value {
        k += 1;
    }

    println!("k: {}", k);

    // Find current directory
    let current_dir = std::env::current_dir()?;
    let current_dir_str = current_dir.to_str().unwrap_or("");

    // Get Home directory - skip binary compilation if TACHYON_DIR is not set (e.g., in tests)
    if let Ok(tachyon_dir) = std::env::var("TACHYON_DIR") {
        let tachyon_dir_str = tachyon_dir.as_str();

        // Run make in the circuit_cpp folder
        info!(LOG, "Compiling circuit binary");
        run_command(
            "bazel-bin/circomlib/build/compile_witness_generator",
            &[
                "--cpp",
                &format!("{}/tmp/circuit_cpp/circuit.cpp", current_dir_str),
            ],
            Some(format!("{}/vendors/circom", tachyon_dir_str).as_str()),
        )
        .await?;

        // Move the binary
        info!(LOG, "Copying binary");
        run_command(
            "mv",
            &[
                "witness_generator",
                &format!("{}/tmp/circuit_cpp/circuit", current_dir_str),
            ],
            Some(format!("{}/vendors/circom", tachyon_dir_str).as_str()),
        )
        .await?;
    } else {
        info!(LOG, "Skipping binary compilation - TACHYON_DIR not set");
    }

    Ok(k as usize)
}

async fn generate_keys(tmp_dir: &str, ptau: usize) -> Result<()> {
    // Generate the proving and verification keys
    info!(LOG, "Downloading ptau file");
    run_command(
        "curl",
        &[
            "-o",
            "pot_final.ptau",
            format!(
                "https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_{}.ptau",
                ptau
            )
            .as_str(),
        ],
        Some(tmp_dir),
    )
    .await?;

    let node_path = run_command_and_return_output("which", &["node"], None)
        .await?
        .trim()
        .to_string();
    let snarkjs_path = run_command_and_return_output("which", &["snarkjs"], None)
        .await?
        .trim()
        .to_string();
    let chunked_snarkjs_path = "./node_modules/.bin/snarkjs";

    println!("node_path: {}", node_path);
    println!("snarkjs_path: {}", snarkjs_path);

    // Generate zkey
    info!(LOG, "Generating zkey");
    run_command(
        &node_path,
        &[
            "--max-old-space-size=65536",
            "--initial-old-space-size=65536",
            "--max-semi-space-size=1024",
            "--initial-heap-size=65536",
            "--expose-gc",
            &snarkjs_path,
            "groth16",
            "setup",
            "circuit.r1cs",
            "pot_final.ptau",
            "circuit_0000.zkey",
        ],
        Some(tmp_dir),
    )
    .await?;

    // Contribute to zkey
    info!(LOG, "Contributing to zkey");
    let random_input: u32 = rand::thread_rng().gen_range(100..1000);
    let random_input_str = format!("{}\n", random_input);
    run_command_with_input(
        "snarkjs",
        &[
            "zkey",
            "beacon",
            "circuit_0000.zkey",
            "circuit_full.zkey",
            "0102030405060708090a0b0c0d0e0f101112231415161718221a1b1c1d1e1f",
            "10",
        ],
        Some(tmp_dir),
        &random_input_str,
    )
    .await?;

    // Generate chunked zkey
    info!(LOG, "Generating chunked zkey");
    run_command(
        &node_path,
        &[
            "--max-old-space-size=65536",
            "--initial-old-space-size=65536",
            "--max-semi-space-size=1024",
            "--initial-heap-size=65536",
            "--expose-gc",
            chunked_snarkjs_path,
            "groth16",
            "setup",
            "circuit.r1cs",
            "pot_final.ptau",
            "circuit_0000.zkey",
        ],
        Some(tmp_dir),
    )
    .await?;

    // Contribute to chunked zkey
    info!(LOG, "Contributing to chunked zkey");
    run_command(
        chunked_snarkjs_path,
        &[
            "zkey",
            "beacon",
            "circuit_0000.zkey",
            "circuit.zkey",
            "0102030405060708090a0b0c0d0e0f101112231415161718221a1b1c1d1e1f",
            "10",
        ],
        Some(tmp_dir),
    )
    .await?;

    run_command("rm", &["pot_final.ptau"], Some(tmp_dir)).await?;
    run_command("rm", &["circuit_0000.zkey"], Some(tmp_dir)).await?;
    for c in ['b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k'] {
        let filename = format!("circuit_0000.zkey{}", c);
        let file_path = Path::new("tmp").join(&filename);
        if file_path.exists() {
            run_command("rm", &[&filename], Some("tmp")).await?;
        }
    }

    // Export verification key
    info!(LOG, "Exporting verification key");
    run_command(
        chunked_snarkjs_path,
        &[
            "zkey",
            "export",
            "verificationkey",
            "circuit.zkey",
            "verification_key.json",
        ],
        Some(tmp_dir),
    )
    .await?;

    Ok(())
}

async fn cleanup() -> Result<()> {
    info!(LOG, "Cleaning up");

    run_command("cp", &["remappings.txt", "./tmp"], None).await?;
    run_command("cp", &["package.json", "./tmp"], None).await?;
    run_command("cp", &["foundry.toml", "./tmp"], None).await?;
    run_command("cp", &["Deploy.s.sol", "./tmp"], None).await?;

    // After generating the chunked zkey, add compression steps
    info!(LOG, "Compressing zkey chunks");
    for c in ['b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k'] {
        let filename = format!("circuit.zkey{}", c);
        if Path::new(&format!("{}/{}", "tmp", filename)).exists() {
            run_command("gzip", &[&filename], Some("tmp")).await?;
        }
    }

    info!(LOG, "Zipping full zkey");
    run_command("gzip", &["circuit.zkey"], Some("tmp")).await?;

    info!(LOG, "Zipping files");
    run_command(
        "zip",
        &[
            "-r",
            "circuit.zip",
            "regex",
            "circuit.circom",
            "Contract.sol",
            "Deploy.s.sol",
            "foundry.toml",
            "package.json",
            "remappings.txt",
            "ClientProofVerifier.sol",
            "ServerProofVerifier.sol",
        ],
        Some("tmp"),
    )
    .await?;

    run_command(
        "zip",
        &["-r", "circuit_cpp.zip", "."],
        Some("tmp/circuit_cpp"),
    )
    .await?;

    run_command("mv", &["circuit_full.zkey", "circuit.zkey"], Some("tmp")).await?;

    run_command(
        "zip",
        &["-r", "circuit_full_zkey.zip", "circuit.zkey"],
        Some("tmp"),
    )
    .await?;

    run_command("mv", &["verification_key.json", "vk.json"], Some("tmp")).await?;

    // Create regex circuit zip file
    info!(LOG, "Creating regex graph zip file");
    run_command(
        "zip",
        &["-r", "circomRegexGraphs.zip", "*_regex.json"],
        Some("tmp/regex/"),
    )
    .await?;

    Ok(())
}

async fn upload_files(upload_urls: UploadUrls) -> Result<()> {
    info!(LOG, "Uploading files");

    // Define required uploads with their paths and content types
    let required_uploads = [
        (&upload_urls.circuit, "./tmp/circuit.zip", "application/zip"),
        (
            &upload_urls.circuit_cpp,
            "./tmp/circuit_cpp/circuit_cpp.zip",
            "application/zip",
        ),
        (
            &upload_urls.circuit_full_zkey,
            "./tmp/circuit_full_zkey.zip",
            "application/zip",
        ),
        (&upload_urls.vk, "./tmp/vk.json", "application/json"),
        (
            &upload_urls.witness_calculator,
            "./tmp/circuit_js/witness_calculator.js",
            "application/octet-stream",
        ),
        (
            &upload_urls.generate_witness,
            "./tmp/circuit_js/generate_witness.js",
            "application/octet-stream",
        ),
        (
            &upload_urls.circuit_wasm,
            "./tmp/circuit_js/circuit.wasm",
            "application/wasm",
        ),
        (
            &upload_urls.circuit_zkey,
            "./tmp/circuit.zkey.gz",
            "application/octet-stream",
        ),
    ];

    // Upload required files if they exist
    for (url, path, content_type) in required_uploads {
        if Path::new(path).exists() {
            upload_to_url(url, path, content_type).await?;
        } else {
            info!(LOG, "Skipping upload for missing file: {}", path);
        }
    }

    // Handle chunked zkey files (b through k)
    let chunked_urls = [
        (&upload_urls.zkey_b, 'b'),
        (&upload_urls.zkey_c, 'c'),
        (&upload_urls.zkey_d, 'd'),
        (&upload_urls.zkey_e, 'e'),
        (&upload_urls.zkey_f, 'f'),
        (&upload_urls.zkey_g, 'g'),
        (&upload_urls.zkey_h, 'h'),
        (&upload_urls.zkey_i, 'i'),
        (&upload_urls.zkey_j, 'j'),
        (&upload_urls.zkey_k, 'k'),
    ];

    for (url, chunk) in chunked_urls {
        let path = format!("./tmp/circuit.zkey{}.gz", chunk);
        if Path::new(&path).exists() {
            upload_to_url(url, &path, "application/octet-stream").await?;
        } else {
            info!(LOG, "Skipping upload for missing chunk: {}", path);
        }
    }

    let regex_json_path = "./tmp/regex/circomRegexGraphs.zip";
    if Path::new(regex_json_path).exists() {
        upload_to_url(
            &upload_urls.circom_regex_graphs,
            regex_json_path,
            "application/zip",
        )
        .await?;
    } else {
        info!(
            LOG,
            "Skipping upload for missing regex JSON file: {}", regex_json_path
        );
    }

    Ok(())
}

pub async fn process_circuit(blueprint: Blueprint) -> Result<usize> {
    setup().await?;

    generate_regex_circuits(blueprint.clone().decomposed_regexes)?;

    let circuit_template_inputs = CircuitTemplateInputs::from(blueprint.clone());

    let circuit = generate_circuit(circuit_template_inputs)?;

    // Write the circuit to a file
    let circuit_path = "./tmp/circuit.circom";
    std::fs::write(circuit_path, circuit)?;

    // Run npm install in the tmp directory to ensure dependencies are available
    info!(LOG, "Installing npm dependencies");
    run_command("npm", &["install"], Some("tmp")).await?;

    let ptau: usize = compile_circuit().await?;

    Ok(ptau)
}

#[cfg(test)]
mod tests {
    use super::*;

    use prost_wkt_types::Timestamp;
    use sdk_utils::proto_types::proto_blueprint::{
        Blueprint, DecomposedRegex, DecomposedRegexPart, ExternalInput,
    };

    #[tokio::test]
    async fn test_compile_circuit_x_export_data() {
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
            client_zk_framework: 1, // Circom
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
                name: "downloadDataLink".to_string(),
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

        // Call the handler with the mock uploader
        let result = process_circuit(blueprint).await;

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }

        // Assert the result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compile_circuit_apple() {
        let blueprint = Blueprint {
            internal_version: "v2".to_string(),
            id: "88802381-0501-4c4a-bcb5-03fdeacf453e".to_string(),
            title: "AppleKYC".to_string(),
            description: "Prove you have a valid Apple account".to_string(),
            slug: "DimiDumo/AppleKYC".to_string(),
            tags: vec![],
            email_query: "from:email.apple.com".to_string(),
            circuit_name: "AppleKYC".to_string(),
            ignore_body_hash_check: true,
            sha_precompute_selector: "".to_string(),
            email_body_max_length: 0,
            sender_domain: "email.apple.com".to_string(),
            enable_header_masking: false,
            enable_body_masking: false,
            client_zk_framework: 1, // Circom
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
                    name: "Subject".to_string(),
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

        let result = process_circuit(blueprint).await;

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }

        // Assert the result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compile_circuit_registry() {
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
            client_zk_framework: 1, // Circom
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
                name: "Subject".to_string(),
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

        let result = process_circuit(blueprint).await;

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }

        // Assert the result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compile_circuit_kraken() {
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
            client_zk_framework: 1, // Circom
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
                name: "EmailSubject".to_string(),
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

        // Call the handler with the mock uploader
        let result = process_circuit(blueprint).await;

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }

        // Assert the result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compile_circuit_subject_extract() {
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
            client_zk_framework: 1, // Circom
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
                name: "Subject".to_string(),
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

        // Call the handler with the mock uploader
        let result = process_circuit(blueprint).await;

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }

        // Assert the result
        assert!(result.is_ok());
    }
}
