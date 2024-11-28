mod contract;
mod db;
mod payload;
mod template;

use std::cmp::max;

use anyhow::Result;
use contract::{
    create_contract, deploy_verifier_contract, generate_verifier_contract, prepare_contract_data,
};
use db::update_db;
use payload::UploadUrls;
use rand::Rng;
use relayer_utils::LOG;
use sdk_utils::{
    run_command, run_command_and_return_output, run_command_with_input, upload_to_url,
};
use slog::info;
use sqlx::postgres::PgPoolOptions;
use template::{generate_circuit, generate_regex_circuits, CircuitTemplateInputs};

#[tokio::main]
async fn main() -> Result<()> {
    let payload = payload::load_payload()?;
    info!(LOG, "Loaded configuration: {:?}", payload);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&payload.database_url)
        .await?;
    println!("Database connection established");

    let blueprint = payload.clone().blueprint;

    setup().await?;

    generate_regex_circuits(blueprint.clone().decomposed_regexes)?;

    let circuit_template_inputs = CircuitTemplateInputs::from(blueprint.clone());

    let circuit = generate_circuit(circuit_template_inputs)?;

    // Write the circuit to a file
    let circuit_path = "./tmp/circuit.circom";
    std::fs::write(circuit_path, circuit)?;

    let ptau: usize = compile_circuit("tmp/circuit.zip").await?;

    println!("ptau: {}", ptau);

    generate_keys("tmp", ptau).await?;

    let contract_data = prepare_contract_data(&payload);

    create_contract(&contract_data)?;

    generate_verifier_contract("tmp").await?;

    let contract_address = deploy_verifier_contract(payload.clone()).await?;

    info!(LOG, "Contract deployed at: {}", contract_address);

    cleanup().await?;

    update_db(
        &pool,
        blueprint.id.expect("No ID found"),
        &contract_address,
        ptau as i32,
    )
    .await?;

    upload_files(payload.upload_url).await?;

    Ok(())
}

async fn setup() -> Result<()> {
    // Remove and recreate tmp and regex directories
    if std::path::Path::new("./tmp").exists() {
        std::fs::remove_dir_all("./tmp")?;
    }
    std::fs::create_dir_all("./tmp")?;

    // Create regex directory inside tmp
    if std::path::Path::new("./tmp/regex").exists() {
        std::fs::remove_dir_all("./tmp/regex")?;
    }
    std::fs::create_dir_all("./tmp/regex")?;

    Ok(())
}

async fn compile_circuit(circuit_path: &str) -> Result<usize> {
    // Run yarn install in the tmp folder
    info!(LOG, "Running yarn install");
    run_command("yarn", &[], Some("tmp")).await?;

    // Compile the circuit
    info!(LOG, "Compiling circuit");
    let compile_result = run_command_and_return_output(
        "circom",
        &[
            "circuit.circom",
            "--sym",
            "--r1cs",
            "--c",
            "--wasm",
            "-l",
            "../node_modules",
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
    while (1u64 << k) <= max_value {
        k += 1;
    }

    println!("k: {}", k);

    // Find current directory
    let current_dir = std::env::current_dir()?;
    let current_dir_str = current_dir.to_str().unwrap_or("");

    // Get Home directory
    let tachyon_dir = std::env::var("TACHYON_DIR")?;
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

    // Generate zkey
    info!(LOG, "Generating zkey");
    run_command(
        "snarkjs",
        &[
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
            "contribute",
            "circuit_0000.zkey",
            "circuit.zkey",
            "-v",
        ],
        Some(tmp_dir),
        &random_input_str,
    )
    .await?;

    // Export verification key
    info!(LOG, "Exporting verification key");
    run_command(
        "snarkjs",
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
    // std::fs::remove_file("./tmp/circuit_0000.zkey")?;
    // std::fs::remove_file("./tmp/pot_final.ptau")?;

    run_command("cp", &["remappings.txt", "./tmp"], None).await?;
    run_command("cp", &["package.json", "./tmp"], None).await?;
    run_command("cp", &["foundry.toml", "./tmp"], None).await?;
    run_command("cp", &["Deploy.s.sol", "./tmp"], None).await?;

    info!(LOG, "Zipping files");
    run_command(
        "zip",
        &[
            "-r",
            "circuit.zip",
            "regex",
            "circuit.circom",
            "contract.sol",
            "Deploy.s.sol",
            "foundry.toml",
            "package.json",
            "remappings.txt",
            "verifier.sol",
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

    run_command(
        "zip",
        &["-r", "circuit_zkey.zip", "circuit.zkey"],
        Some("tmp"),
    )
    .await?;

    run_command("mv", &["verification_key.json", "vk.json"], Some("tmp")).await?;

    Ok(())
}

async fn upload_files(upload_urls: UploadUrls) -> Result<()> {
    info!(LOG, "Uploading files");

    upload_to_url(&upload_urls.circuit, "./tmp/circuit.zip", "application/zip").await?;
    upload_to_url(
        &upload_urls.circuit_cpp,
        "./tmp/circuit_cpp.zip",
        "application/zip",
    )
    .await?;
    upload_to_url(
        &upload_urls.circuit_zkey,
        "./tmp/circuit_zkey.zip",
        "application/zip",
    )
    .await?;
    upload_to_url(&upload_urls.vk, "./tmp/vk.json", "application/json").await?;
    upload_to_url(
        &upload_urls.witness_calculator,
        "./tmp/circuit_js/witness_calculator.js",
        "application/octet-stream",
    )
    .await?;
    upload_to_url(
        &upload_urls.circuit_wasm,
        "./tmp/circuit_js/circuit.wasm",
        "application/wasm",
    )
    .await?;

    Ok(())
}
