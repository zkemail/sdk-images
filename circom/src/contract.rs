use std::{collections::HashMap, fs, path::Path};

use anyhow::Result;
use regex::Regex;
use relayer_utils::LOG;
use sdk_utils::{run_command, run_command_and_capture_output};
use serde::{Deserialize, Serialize};
use slog::info;
use tera::{Context, Tera};

use crate::payload::Payload;

#[derive(Serialize, Deserialize)]
pub struct ContractData {
    pub sender_domain: String,
    pub values: Vec<Field>,
    pub external_inputs: Vec<Field>,
    pub signal_size: usize,
    pub prover_eth_address_idx: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub max_length: usize,
    pub pack_size: usize,
    pub start_idx: usize,
}

/// Render the Solidity ZKEmailVerifier contract template and write it to the given path.
pub fn create_zkemail_verifier_contract_at_path(
    contract_data: &ContractData,
    output_path: &str,
) -> Result<()> {
    // Initialize Tera
    let mut tera = Tera::default();
    tera.add_template_file("./templates/ZKEmailVerifier.sol.tera", Some("Contract.sol"))?;

    let mut context = Context::new();
    context.insert("sender_domain", &contract_data.sender_domain);
    context.insert("values", &contract_data.values);
    context.insert("external_inputs", &contract_data.external_inputs);
    context.insert("signal_size", &contract_data.signal_size);
    context.insert(
        "prover_eth_address_idx",
        &contract_data.prover_eth_address_idx,
    );

    let rendered_contract = tera.render("Contract.sol", &context)?;

    // Ensure parent directory exists, then write
    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(output_path, rendered_contract)?;

    Ok(())
}

/// Render the Solidity mock Groth16Verifier contract template and write it to the given path.
pub fn create_mock_groth16_verifier_at_path(
    contract_data: &ContractData,
    output_path: &str,
) -> Result<()> {
    let mut tera = Tera::default();
    tera.add_template_file(
        "./templates/MockGroth16Verifier.sol.tera",
        Some("Groth16Verifier.sol"),
    )?;

    let mut context = Context::new();
    context.insert("signal_size", &contract_data.signal_size);

    let rendered_contract = tera.render("Groth16Verifier.sol", &context)?;

    std::fs::write(output_path, rendered_contract)?;

    Ok(())
}

pub fn prepare_contract_data(payload: &Payload) -> ContractData {
    let mut signal_size = 1 + 1 + 2; // For pubkey, proverETHAddress and sha256 hash of header
    let mut current_idx = 1;

    let mut values = Vec::new();
    for regex in &payload.blueprint.decomposed_regexes {
        let pack_size = ((regex.max_match_length as f64) / 31.0).ceil() as usize;
        let field = Field {
            name: regex.name.clone(),
            max_length: regex.max_match_length as usize,
            pack_size,
            start_idx: current_idx,
        };
        for part in regex.parts.iter() {
            if part.is_public == Some(true) {
                if regex.is_hashed.unwrap_or(false) {
                    signal_size += 1;
                    current_idx += 1;
                } else {
                    signal_size += pack_size;
                    current_idx += pack_size;
                }
            }
        }
        values.push(field);
    }

    let prover_eth_address_idx = current_idx;
    current_idx += 1; // Add 1 prover ETH address

    let mut external_inputs = Vec::new();
    for input in &payload.blueprint.external_inputs {
        let pack_size = ((input.max_length as f64) / 31.0).ceil() as usize;
        let field = Field {
            name: input.name.clone(),
            max_length: input.max_length as usize,
            pack_size,
            start_idx: current_idx,
        };
        signal_size += pack_size;
        current_idx += pack_size;
        external_inputs.push(field);
    }

    ContractData {
        sender_domain: payload.blueprint.sender_domain.clone(),
        values,
        external_inputs,
        signal_size,
        prover_eth_address_idx,
    }
}

/// Generate a Groth16 verifier contract from a zkey and write it to the given output path.
/// The contract name in the Solidity source is derived from the output filename (e.g. Groth16Verifier.sol -> Groth16Verifier).
pub async fn generate_verifier_contract(
    tmp_dir: &str,
    snarkjs_path: &str,
    zkey_file_name: &str,
    output_path: &str,
) -> Result<()> {
    // Generate the verifier contract (snarkjs writes verifier.sol to tmp_dir)
    info!(LOG, "Generating verifier contract");
    run_command(
        snarkjs_path,
        &[
            "zkey",
            "export",
            "solidityverifier",
            zkey_file_name,
            "verifier.sol",
        ],
        Some(tmp_dir),
    )
    .await?;

    let verifier_path = Path::new(tmp_dir).join("verifier.sol");
    let output = Path::new(output_path);

    // Derive contract name from output filename (e.g. Groth16Verifier.sol -> Groth16Verifier)
    let contract_name = output
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Groth16Verifier");

    // Read the verifier contract
    let content = fs::read_to_string(&verifier_path)?;
    // Patch version and rename the contract
    let updated_content = content
        .replace(
            Regex::new(r"pragma solidity .*;")
                .unwrap()
                .find(&content)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Could not find pragma solidity declaration in verifier contract"
                    )
                })?
                .as_str(),
            &format!("pragma solidity ^{};", "0.8.34"),
        )
        .replace(
            Regex::new(r"contract .*\{")
                .unwrap()
                .find(&content)
                .ok_or_else(|| {
                    anyhow::anyhow!("Could not find contract declaration in verifier contract")
                })?
                .as_str(),
            &format!("contract {} {{", contract_name),
        );

    // Ensure parent directory exists, then write
    if let Some(parent) = output.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(output, updated_content)?;
    fs::remove_file(&verifier_path)?;

    info!(
        LOG,
        "Wrote verifier to {} (contract {})", output_path, contract_name
    );

    Ok(())
}

const ONCHAIN_SUCCESS_MARKER: &str = "ONCHAIN EXECUTION COMPLETE & SUCCESSFUL";
const VERIFICATION_FAILURE_MARKER: &str = "contracts were verified!";

/// Parses deploy script output: returns ZK_EMAIL_VERIFIER address if onchain execution succeeded,
/// and logs when verification failed. Used so verification failure does not fail the deploy.
fn parse_deploy_output(output: &str) -> Result<String> {
    if !output.contains(ONCHAIN_SUCCESS_MARKER) {
        return Err(anyhow::anyhow!(
            "Deploy failed: output did not contain '{}'",
            ONCHAIN_SUCCESS_MARKER
        ));
    }

    if output.contains("Error: Not all") && output.contains(VERIFICATION_FAILURE_MARKER) {
        info!(LOG, "Deploy successful, verification failed");
    }

    let re = Regex::new(r"(DKIM_REGISTRY|GROTH16_VERIFIER|ZK_EMAIL_VERIFIER): (0x[a-fA-F0-9]{40})")
        .unwrap();
    let mut contract_addresses = HashMap::new();
    for cap in re.captures_iter(output) {
        let contract_name = &cap[1];
        let address = &cap[2];
        contract_addresses.insert(contract_name.to_string(), address.to_string());
        info!(LOG, "{} Contract is at: {}", contract_name, address);
    }

    contract_addresses
        .get("ZK_EMAIL_VERIFIER")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("ZK_EMAIL_VERIFIER address not found in deploy output"))
}

pub async fn deploy_verifier_contract(_payload: Payload) -> Result<String> {
    let contracts_dir = "tmp/contracts";

    info!(LOG, "Building contracts");
    run_command("yarn", &["build"], Some(contracts_dir)).await?;

    info!(LOG, "Deploying contracts");
    let (output, _success) = tokio::task::spawn_blocking(|| {
        run_command_and_capture_output("yarn", &["deploy"], Some(contracts_dir))
    })
    .await
    .map_err(|e| anyhow::anyhow!("deploy task: {:?}", e))??;

    parse_deploy_output(&output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_deploy_output_success_with_verification_failure() {
        // Simulates real forge output: deploy to Anvil succeeded, Etherscan verification failed
        let output = r#"DKIM_REGISTRY: 0xf825ac88B042e9E31Ce18bAaE18EF8c31Eae7C8f
GROTH16_VERIFIER: 0x3e30c437DC8f92e4B631b7b0BFdE00fb30C11fD5
ZK_EMAIL_VERIFIER: 0xE03267033B1606d2FDe825B35ec66dFD66E442AD

ONCHAIN EXECUTION COMPLETE & SUCCESSFUL.
Error: Not all (0 / 2) contracts were verified!"#;
        let addr = parse_deploy_output(output).unwrap();
        assert_eq!(addr, "0xE03267033B1606d2FDe825B35ec66dFD66E442AD");
    }

    #[test]
    fn parse_deploy_output_fails_without_onchain_success() {
        let output = "Script ran but no ONCHAIN EXECUTION line";
        assert!(parse_deploy_output(output).is_err());
    }

    #[test]
    fn parse_deploy_output_success_without_verification_failure() {
        let output = r#"ZK_EMAIL_VERIFIER: 0xE03267033B1606d2FDe825B35ec66dFD66E442AD
ONCHAIN EXECUTION COMPLETE & SUCCESSFUL."#;
        let addr = parse_deploy_output(output).unwrap();
        assert_eq!(addr, "0xE03267033B1606d2FDe825B35ec66dFD66E442AD");
    }
}
