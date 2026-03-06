use anyhow::{Result, anyhow};
use regex::Regex;
use relayer_utils::LOG;
use sdk_utils::proto_types::proto_blueprint::Blueprint;
use serde::Deserialize;
use slog::{info, warn};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Uuid;
use std::path::{Path, PathBuf};

use crate::circuit_pipeline::{CompiledCircuit, build_circuit_artifacts};
use crate::db::update_verifier_contract_address;
use crate::external_command::{
    run_yarn_build, run_yarn_build_polka, run_yarn_deploy, run_yarn_deploy_polka, run_yarn_verify,
    run_yarn_verify_polka,
};
use crate::filesystem::{FileUploader, UploadTarget, zip_circuit_dir, zip_regex_graphs};
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

/// Result of a successful blueprint compile before any packaging or uploads.
/// Returned by `compile_blueprint_artifacts`; pass to
/// `package_blueprint_artifacts` to create zipped artifacts, and to
/// `deploy_blueprint_contracts` to deploy contracts (optional).
#[derive(Debug, Clone)]
pub struct CompiledBlueprint {
    pub artifacts_1024: CompiledCircuit,
    pub artifacts_2048: CompiledCircuit,
    /// Directory containing the generated regex graphs Noir project.
    pub regex_graphs_dir: PathBuf,
}

/// Result of packaging a compiled blueprint into zipped artifacts ready for
/// upload.
#[derive(Debug, Clone)]
pub struct PackagedBlueprint {
    pub compiled: CompiledBlueprint,
    pub circuit_1024_zip: PathBuf,
    pub circuit_2048_zip: PathBuf,
    pub regex_graphs_zip: PathBuf,
}

/// Internal setup for the top-level `compile_blueprint` call. Cleans the given
/// tmp directory (removes everything inside it), then creates the subdirectories:
/// - `tmp/regex_graphs`
/// - `tmp/1024`
/// - `tmp/2048`
fn compile_blueprint_setup(tmp_dir: &Path) -> Result<(PathBuf, PathBuf, PathBuf, PathBuf)> {
    if tmp_dir.exists() {
        for entry in std::fs::read_dir(tmp_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                std::fs::remove_dir_all(&path)?;
            } else {
                std::fs::remove_file(&path)?;
            }
        }
    } else {
        std::fs::create_dir_all(tmp_dir)?;
    }

    let regex_graphs_dir = tmp_dir.join("regex_graphs");
    std::fs::create_dir_all(&regex_graphs_dir)?;

    let key_1024_dir = tmp_dir.join("1024");
    let key_2048_dir = tmp_dir.join("2048");
    std::fs::create_dir_all(&key_1024_dir)?;
    std::fs::create_dir_all(&key_2048_dir)?;

    Ok((
        tmp_dir.to_path_buf(),
        regex_graphs_dir,
        key_1024_dir,
        key_2048_dir,
    ))
}

/// Compiles a single blueprint without performing any packaging, uploads, or
/// deployments. This includes:
/// - filesystem setup
/// - regex circuit generation
/// - per-key-size circuit generation and compilation
pub async fn compile_blueprint_artifacts(
    tmp_dir: &Path,
    payload: &Payload,
) -> Result<CompiledBlueprint> {
    let blueprint = payload.blueprint.clone();

    // Setup filesystem and top-level directories used by the pipeline.
    let (tmp_dir, _regex_graphs_dir, key_1024_dir, key_2048_dir) =
        compile_blueprint_setup(tmp_dir)?;

    // Generate regex circuits (shared between both key sizes) under this tmp dir.
    // The generator will populate `tmp/regex_graphs` with Noir + JSON artifacts.
    let regex_graphs_dir = generate_regex_circuits(&tmp_dir, &blueprint.decomposed_regexes)?;

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

    // Build full artifact bundles for 1024-bit and 2048-bit circuits.
    Ok(CompiledBlueprint {
        artifacts_1024: build_circuit_artifacts(&key_1024_dir, &blueprint, 1024, &regex_graphs_dir)
            .await?,
        artifacts_2048: build_circuit_artifacts(&key_2048_dir, &blueprint, 2048, &regex_graphs_dir)
            .await?,
        regex_graphs_dir,
    })
}

/// Packages compiled blueprint artifacts into zipped files ready for upload.
/// This is a pure filesystem operation.
pub async fn package_blueprint_artifacts(
    tmp_dir: &Path,
    compiled: &CompiledBlueprint,
) -> Result<PackagedBlueprint> {
    let circuit_1024_zip =
        zip_circuit_dir(&tmp_dir.join("1024"), &tmp_dir.join("circuit_1024.zip")).await?;
    let circuit_2048_zip =
        zip_circuit_dir(&tmp_dir.join("2048"), &tmp_dir.join("circuit_2048.zip")).await?;
    let regex_graphs_zip = zip_regex_graphs(&compiled.regex_graphs_dir, "regex_graphs.zip").await?;

    Ok(PackagedBlueprint {
        compiled: compiled.clone(),
        circuit_1024_zip,
        circuit_2048_zip,
        regex_graphs_zip,
    })
}

/// Uploads packaged blueprint artifacts (circuits + regex graphs) using the
/// provided uploader. This is purely an upload step and performs no compile,
/// packaging, or deployment work.
pub async fn upload_blueprint_artifacts(
    artifacts: &PackagedBlueprint,
    upload_urls: &UploadUrls,
    uploader: impl FileUploader,
) -> Result<()> {
    let to_string = |p: &Path| p.to_string_lossy().into_owned();

    let upload_targets = vec![
        UploadTarget {
            url: upload_urls.circuit_1024.clone(),
            path: to_string(&artifacts.circuit_1024_zip),
            content_type: "application/zip".to_string(),
        },
        UploadTarget {
            url: upload_urls.circuit_2048.clone(),
            path: to_string(&artifacts.circuit_2048_zip),
            content_type: "application/zip".to_string(),
        },
        UploadTarget {
            url: upload_urls.circuit_json_1024.clone(),
            path: to_string(&artifacts.compiled.artifacts_1024.bytecode_path),
            content_type: "application/json".to_string(),
        },
        UploadTarget {
            url: upload_urls.circuit_json_2048.clone(),
            path: to_string(&artifacts.compiled.artifacts_2048.bytecode_path),
            content_type: "application/json".to_string(),
        },
        UploadTarget {
            url: upload_urls.regex_graphs.clone(),
            path: to_string(&artifacts.regex_graphs_zip),
            content_type: "application/zip".to_string(),
        },
    ];

    uploader.upload_files(upload_targets).await
}

/// Deploys Foundry contracts for both 1024- and 2048-bit circuits, parses the
/// `ZK_EMAIL_VERIFIER` address from each deploy output, and updates the
/// database. The caller should check `should_deploy` before calling this.
pub async fn deploy_blueprint_contracts(
    compiled: &CompiledBlueprint,
    payload: &Payload,
) -> Result<()> {
    let addr_1024 = deploy_contracts_for_circuit(&compiled.artifacts_1024, payload).await?;
    let addr_2048 = deploy_contracts_for_circuit(&compiled.artifacts_2048, payload).await?;

    // Prefer the 2048-bit address since 2048-bit RSA keys are more common for
    // DKIM; fall back to the 1024-bit address.
    let address = addr_2048.or(addr_1024);

    if let Some(ref addr) = address {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&payload.database_url)
            .await?;

        let blueprint_id = Uuid::parse_str(&payload.blueprint.id)?;
        update_verifier_contract_address(&pool, blueprint_id, addr).await?;
        info!(
            LOG,
            "Updated verifier_contract_address in DB for blueprint {}: {}",
            payload.blueprint.id,
            addr
        );
    } else {
        info!(
            LOG,
            "Could not parse ZK_EMAIL_VERIFIER address from deploy output; skipping DB update"
        );
    }

    Ok(())
}

/// Deploys the contracts package for a single circuit directory by running
/// `yarn deploy` (EVM) or `yarn deploy:polka` (Polkadot) inside the
/// `<circuit_dir>/contracts` directory.
///
/// Returns the `ZK_EMAIL_VERIFIER` contract address parsed from the deploy
/// output, or `None` if parsing failed.
async fn deploy_contracts_for_circuit(
    artifacts: &CompiledCircuit,
    payload: &Payload,
) -> Result<Option<String>> {
    let contracts_dir = &artifacts.contracts_dir;
    if !contracts_dir.exists() {
        return Err(anyhow!(
            "Expected contracts directory at '{}' but it does not exist",
            contracts_dir.display()
        ));
    }

    let contracts_dir_str = contracts_dir.to_str().ok_or_else(|| {
        anyhow!(
            "Contracts directory path '{}' is not valid UTF-8",
            contracts_dir.display()
        )
    })?;

    let chain_id_str = payload.chain_id.to_string();
    let envs: &[(&str, &str)] = &[
        ("PRIVATE_KEY", payload.private_key.as_str()),
        ("RPC_URL", payload.rpc_url.as_str()),
        ("CHAIN_ID", chain_id_str.as_str()),
        ("DKIM_REGISTRY", payload.dkim_registry_address.as_str()),
        ("ETHERSCAN_API_KEY", payload.etherscan_api_key.as_str()),
    ];

    const POLKADOT_HUB_TESTNET_CHAIN_ID: u32 = 420420417;
    let deploy_output = if payload.chain_id == POLKADOT_HUB_TESTNET_CHAIN_ID {
        run_yarn_build_polka(contracts_dir_str).await?;
        let output = run_yarn_deploy_polka(contracts_dir_str, envs).await?;
        if let Err(e) = run_yarn_verify_polka(contracts_dir_str, envs).await {
            warn!(LOG, "Contract verification failed (polka): {}", e);
        }
        output
    } else {
        run_yarn_build(contracts_dir_str).await?;
        let output = run_yarn_deploy(contracts_dir_str, envs).await?;
        // ETHERSCAN_API_KEY is used as a generic gate for contract verification,
        // including Blockscout-based chains like Polkadot Hub (see hardhat.config.ts).
        if !payload.etherscan_api_key.trim().is_empty() {
            if let Err(e) = run_yarn_verify(contracts_dir_str, envs).await {
                warn!(LOG, "Contract verification failed: {}", e);
            }
        } else {
            info!(LOG, "Skipping contract verification: no ETHERSCAN_API_KEY");
        }
        output
    };

    let address = parse_zk_email_verifier_address(&deploy_output);
    if let Some(ref addr) = address {
        info!(LOG, "Parsed ZK_EMAIL_VERIFIER address: {}", addr);
    }

    Ok(address)
}

/// Extracts the `ZK_EMAIL_VERIFIER` contract address from deployment output.
/// Both the Forge and Hardhat deploy scripts print a line like:
///   `ZK_EMAIL_VERIFIER: 0x<40 hex chars>`
fn parse_zk_email_verifier_address(output: &str) -> Option<String> {
    let re = Regex::new(r"ZK_EMAIL_VERIFIER:\s*(0x[a-fA-F0-9]{40})").ok()?;
    re.captures(output)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}
