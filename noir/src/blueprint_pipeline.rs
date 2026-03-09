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
    run_yarn_build, run_yarn_build_polka, run_yarn_deploy, run_yarn_deploy_polka, run_yarn_install,
    run_yarn_verify, run_yarn_verify_polka,
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

#[derive(Deserialize, Clone)]
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

impl std::fmt::Debug for Payload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Payload")
            .field("blueprint", &self.blueprint)
            .field("upload_urls", &"[REDACTED]")
            .field("database_url", &"[REDACTED]")
            .field("private_key", &"[REDACTED]")
            .field("rpc_url", &"[REDACTED]")
            .field("chain_id", &self.chain_id)
            .field("etherscan_api_key", &"[REDACTED]")
            .field("dkim_registry_address", &self.dkim_registry_address)
            .finish()
    }
}

/// Sensitive configuration extracted from the payload for contract deployment.
/// Passed through Rust code and only set as env vars at child-process spawn
/// via `cmd.env()`, avoiding process-wide mutation that would race under
/// concurrent requests.
pub struct DeployConfig {
    pub database_url: String,
    pub private_key: String,
    pub rpc_url: String,
    pub chain_id: u32,
    pub etherscan_api_key: String,
    pub dkim_registry_address: String,
}

impl std::fmt::Debug for DeployConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeployConfig")
            .field("database_url", &"[REDACTED]")
            .field("private_key", &"[REDACTED]")
            .field("rpc_url", &"[REDACTED]")
            .field("chain_id", &self.chain_id)
            .field("etherscan_api_key", &"[REDACTED]")
            .field("dkim_registry_address", &self.dkim_registry_address)
            .finish()
    }
}

impl DeployConfig {
    pub fn should_deploy(&self) -> bool {
        !self.private_key.trim().is_empty()
            && !self.rpc_url.trim().is_empty()
            && !self.dkim_registry_address.trim().is_empty()
    }

    pub fn as_env_pairs(&self) -> Vec<(String, String)> {
        vec![
            ("PRIVATE_KEY".into(), self.private_key.clone()),
            ("RPC_URL".into(), self.rpc_url.clone()),
            ("CHAIN_ID".into(), self.chain_id.to_string()),
            ("DKIM_REGISTRY".into(), self.dkim_registry_address.clone()),
            ("ETHERSCAN_API_KEY".into(), self.etherscan_api_key.clone()),
        ]
    }
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
    blueprint: &Blueprint,
) -> Result<CompiledBlueprint> {
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
        artifacts_1024: build_circuit_artifacts(&key_1024_dir, blueprint, 1024, &regex_graphs_dir)
            .await?,
        artifacts_2048: build_circuit_artifacts(&key_2048_dir, blueprint, 2048, &regex_graphs_dir)
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
    let regex_graphs_zip = zip_regex_graphs(
        &compiled.regex_graphs_dir,
        &tmp_dir.join("regex_graphs.zip"),
    )
    .await?;

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
/// database. Checks `config.should_deploy()` internally and returns early
/// when required fields are missing.
pub async fn deploy_blueprint_contracts(
    compiled: &CompiledBlueprint,
    config: &DeployConfig,
    blueprint_id: &str,
) -> Result<()> {
    if !config.should_deploy() {
        info!(LOG, "Skipping contract deployment: missing required config");
        return Ok(());
    }

    let addr_1024 = deploy_contracts_for_circuit(&compiled.artifacts_1024, config).await?;
    let addr_2048 = deploy_contracts_for_circuit(&compiled.artifacts_2048, config).await?;

    // Prefer the 2048-bit address since 2048-bit RSA keys are more common for
    // DKIM; fall back to the 1024-bit address.
    let address = addr_2048.or(addr_1024);

    if let Some(ref addr) = address {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&config.database_url)
            .await?;

        let blueprint_uuid = Uuid::parse_str(blueprint_id)?;
        update_verifier_contract_address(&pool, blueprint_uuid, addr).await?;
        info!(
            LOG,
            "Updated verifier_contract_address in DB for blueprint {}: {}", blueprint_id, addr
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
    config: &DeployConfig,
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

    // Ensure node_modules are installed for this per-circuit contracts directory
    run_yarn_install(contracts_dir_str).await?;

    let env_pairs = config.as_env_pairs();
    let envs: Vec<(&str, &str)> = env_pairs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    const POLKADOT_HUB_TESTNET_CHAIN_ID: u32 = 420420417;
    let deploy_output = if config.chain_id == POLKADOT_HUB_TESTNET_CHAIN_ID {
        run_yarn_build_polka(contracts_dir_str).await?;
        let output = run_yarn_deploy_polka(contracts_dir_str, &envs).await?;
        if let Err(e) = run_yarn_verify_polka(contracts_dir_str, &envs).await {
            warn!(LOG, "Contract verification failed (polka): {}", e);
        }
        output
    } else {
        run_yarn_build(contracts_dir_str).await?;
        let output = run_yarn_deploy(contracts_dir_str, &envs).await?;
        // ETHERSCAN_API_KEY is used as a generic gate for contract verification,
        // including Blockscout-based chains like Polkadot Hub (see hardhat.config.ts).
        if !config.etherscan_api_key.trim().is_empty() {
            if let Err(e) = run_yarn_verify(contracts_dir_str, &envs).await {
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
