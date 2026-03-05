use anyhow::{Result, anyhow};
use relayer_utils::LOG;
use sdk_utils::{run_command, run_command_with_env};
use slog::info;
use std::path::Path;

/// Supported oracle hash algorithms for `bb write_vk`.
#[derive(Debug, Clone, Copy)]
pub enum OracleHash {
    Poseidon2,
    Keccak,
}

impl OracleHash {
    pub fn as_str(&self) -> &'static str {
        match self {
            OracleHash::Poseidon2 => "poseidon2",
            OracleHash::Keccak => "keccak",
        }
    }
}

/// Runs `nargo compile` in the provided working directory and returns:
/// - the `target` directory under `workdir`
/// - the path to the generated `target/<package_name>.json`.
/// Fails if the expected bytecode file is not present after compilation.
pub async fn run_nargo_compile(
    workdir: &Path,
    package_name: &str,
) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    let workdir_str = workdir
        .to_str()
        .ok_or_else(|| anyhow!("compile_circuit cwd must be valid UTF-8"))?;
    info!(
        LOG,
        "Compiling circuit in {} for package {}", workdir_str, package_name
    );
    run_command(
        "nargo",
        &["compile", "--package", package_name],
        Some(workdir_str),
    )
    .await?;

    let target_dir = workdir.join("target");
    let bytecode_file = format!("{package_name}.json");
    let bytecode_path = target_dir.join(bytecode_file);
    if !bytecode_path.exists() {
        return Err(anyhow!(
            "Expected circuit bytecode at '{}' after nargo compile, but it does not exist",
            bytecode_path.display()
        ));
    }

    Ok((target_dir, bytecode_path))
}

/// Runs `bb write_vk` to derive the verification key for the circuit in
/// the provided working directory.
///
/// - `bytecode_path`: full path to the circuit bytecode JSON
/// - `output_dir_path`: full path to the directory where the VK will be written
/// - `oracle_hash`: oracle hash algorithm to use (e.g. "keccak")
pub async fn run_bb_write_vk(
    workdir: &Path,
    bytecode_path: &Path,
    output_dir_path: &Path,
    oracle_hash: OracleHash,
) -> Result<std::path::PathBuf> {
    let bytecode_path_str = bytecode_path
        .to_str()
        .ok_or_else(|| anyhow!("bytecode_path must be valid UTF-8"))?;
    let output_path_str = output_dir_path
        .to_str()
        .ok_or_else(|| anyhow!("output_path must be valid UTF-8"))?;

    let workdir_str = workdir
        .to_str()
        .ok_or_else(|| anyhow!("compile_circuit cwd must be valid UTF-8"))?;

    info!(LOG, "Writing verification key");
    run_command(
        "bb",
        &[
            "write_vk",
            "--bytecode_path",
            bytecode_path_str,
            "--output_path",
            output_path_str,
            "--oracle_hash",
            oracle_hash.as_str(),
        ],
        Some(workdir_str),
    )
    .await?;

    // Verify that the expected verification key file was produced and return
    // its full path (`output_dir_path`/`vk`).
    let vk_path = output_dir_path.join("vk");
    if !vk_path.exists() {
        return Err(anyhow!(
            "Expected verification key at '{}' after bb write_vk, but it does not exist",
            vk_path.display()
        ));
    }

    Ok(vk_path)
}

/// Runs `bb write_solidity_verifier` to generate the Solidity Honk verifier
/// contract for the circuit in the provided working directory, and returns
/// the verifier contract path.
///
/// - `vk_path`: full path to the verification key
/// - `output_file_path`: full path where the Solidity verifier contract
///   will be written.
pub async fn run_bb_write_solidity_verifier(
    workdir: &Path,
    vk_path: &Path,
    output_file_path: &Path,
) -> Result<std::path::PathBuf> {
    let vk_path_str = vk_path
        .to_str()
        .ok_or_else(|| anyhow!("vk_path must be valid UTF-8"))?;
    let output_path_str = output_file_path
        .to_str()
        .ok_or_else(|| anyhow!("output_path must be valid UTF-8"))?;
    let workdir_str = workdir
        .to_str()
        .ok_or_else(|| anyhow!("compile_circuit cwd must be valid UTF-8"))?;

    info!(LOG, "Writing Solidity Honk verifier");
    run_command(
        "bb",
        &[
            "write_solidity_verifier",
            "--vk_path",
            vk_path_str,
            "--output_path",
            output_path_str,
        ],
        Some(workdir_str),
    )
    .await?;

    // Verify that the expected Solidity verifier file was produced and return
    // its full path.
    if !output_file_path.exists() {
        return Err(anyhow!(
            "Expected Solidity Honk verifier at '{}' after bb write_solidity_verifier, but it does not exist",
            output_file_path.display()
        ));
    }

    Ok(output_file_path.to_path_buf())
}

/// Runs `yarn build` in the given contracts directory (EVM / Foundry).
pub async fn run_yarn_build(contracts_dir: &str) -> Result<()> {
    info!(LOG, "Building contracts in {} using yarn build", contracts_dir);
    run_command("yarn", &["build"], Some(contracts_dir)).await
}

/// Runs `yarn build:polka` in the given contracts directory (Polkadot / Hardhat).
pub async fn run_yarn_build_polka(contracts_dir: &str) -> Result<()> {
    info!(
        LOG,
        "Building contracts in {} using yarn build:polka", contracts_dir
    );
    run_command("yarn", &["build:polka"], Some(contracts_dir)).await
}

/// Runs `yarn deploy` in the given contracts directory with the provided
/// environment variables (EVM / Foundry deployment).
pub async fn run_yarn_deploy(
    contracts_dir: &str,
    envs: &[(&str, &str)],
) -> Result<()> {
    info!(LOG, "Deploying contracts from {} using yarn deploy", contracts_dir);
    run_command_with_env("yarn", &["deploy"], Some(contracts_dir), envs).await
}

/// Runs `yarn deploy:polka` in the given contracts directory with the provided
/// environment variables (Polkadot deployment).
pub async fn run_yarn_deploy_polka(
    contracts_dir: &str,
    envs: &[(&str, &str)],
) -> Result<()> {
    info!(
        LOG,
        "Deploying contracts from {} using yarn deploy:polka", contracts_dir
    );
    run_command_with_env("yarn", &["deploy:polka"], Some(contracts_dir), envs).await
}

