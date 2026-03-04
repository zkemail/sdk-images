//! Lightweight CLI subcommands that run without starting the compile API server.

use anyhow::Result;
use std::path::PathBuf;

use crate::filesystem::{create_example_contracts_at_path, ExampleContractData};

/// If `args` matches a known CLI subcommand, run it and return `Some(result)`.
/// Otherwise return `None` so the caller can start the server.
pub fn run_if_cli(args: &[String]) -> Option<Result<()>> {
    if args.len() >= 2 && args[1] == "generate-example-contracts" {
        return Some(run_generate_example_contracts(args));
    }
    None
}

/// Usage: noir generate-example-contracts <contract_data_json_path> [<output_dir>]
/// JSON format: { "senderDomain": "example.com", "publicInputsLength": 2 }
/// If <output_dir> is omitted, files are written to <noir_crate_root>/contracts/src/
fn run_generate_example_contracts(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        anyhow::bail!(
            "Usage: noir generate-example-contracts <contract_data_json_path> [<output_dir>]"
        );
    }
    let json_path = &args[2];
    let output_dir: PathBuf = if args.len() >= 4 {
        PathBuf::from(&args[3])
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("contracts")
            .join("src")
    };

    let json = std::fs::read_to_string(json_path).map_err(|e| {
        anyhow::anyhow!("Failed to read contract data from {}: {}", json_path, e)
    })?;
    let contract_data: ExampleContractData = serde_json::from_str(&json)
        .map_err(|e| anyhow::anyhow!("Invalid contract data JSON: {}", e))?;

    create_example_contracts_at_path(&contract_data, &output_dir)?;

    println!(
        "Populated HonkVerifier.sol and ZKEmailVerifier.sol written to {}",
        output_dir.display()
    );

    Ok(())
}
