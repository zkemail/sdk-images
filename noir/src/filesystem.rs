use anyhow::{Result, anyhow};
use regex::Regex;
use relayer_utils::LOG;
use sdk_utils::{run_command, upload_to_url};
use slog::info;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg_attr(test, mockall::automock)]
pub trait FileUploader {
    fn upload_files(&self, targets: Vec<UploadTarget>) -> impl Future<Output = Result<()>> + Send;
}

/// Describes a single file upload: where to upload (`url`), which local file
/// to read (`path`), and the HTTP content type to use (`content_type`).
#[derive(Debug, Clone)]
pub struct UploadTarget {
    pub url: String,
    pub path: String,
    pub content_type: String,
}

pub struct ProductionFileUploader;

impl FileUploader for ProductionFileUploader {
    async fn upload_files(&self, targets: Vec<UploadTarget>) -> Result<()> {
        for target in targets {
            upload_to_url(&target.url, &target.path, &target.content_type).await?;
        }

        Ok(())
    }
}

/// Zips the Noir project at `circuit_dir` (and its sibling `contracts` folder)
/// into `zip_name` under the shared tmp dir and returns the full path to the
/// created zip file.
pub async fn zip_circuit_dir(cwd: &Path, zip_path: &Path) -> Result<PathBuf> {
    info!(
        LOG,
        "Zipping circuit Noir project (noir + contracts) to {}",
        zip_path.display()
    );

    const FILES_TO_ZIP: &[&str] = &[
        // noir
        "noir/Nargo.toml",
        "noir/src",
        // contracts shared config
        "contracts/.env.example",
        "contracts/package.json",
        "contracts/README.md",
        "contracts/yarn.lock",
        // contracts foundry
        "contracts/foundry.toml",
        "contracts/remappings.txt",
        "contracts/script/DeployZKEmailVerifier.s.sol",
        // contracts hardhat
        "contracts/hh-scripts/deploy-zk-email-verifier.ts",
        "contracts/utils/requireEnv.ts",
        "contracts/hardhat.config.ts",
        "contracts/tsconfig.json",
        // contracts contracts
        "contracts/src/interfaces/IDKIMRegistry.sol",
        "contracts/src/interfaces/IHonkVerifier.sol",
        "contracts/src/interfaces/IZKEmailVerifier.sol",
        "contracts/src/HonkVerifier.sol",
        "contracts/src/ZKEmailVerifier.sol",
    ];

    let out_str = zip_path
        .to_str()
        .ok_or_else(|| anyhow!("zip_path must be valid UTF-8"))?;
    let cwd_str = cwd
        .to_str()
        .ok_or_else(|| anyhow!("cwd must be valid UTF-8"))?;
    let mut args = vec!["-r", out_str];
    args.extend(FILES_TO_ZIP);
    run_command("zip", &args, Some(cwd_str)).await?;

    Ok(zip_path.to_path_buf())
}

/// Zips regex graphs into `zip_name` under `holder_dir` and returns
/// the full path to the created zip file.
pub async fn zip_regex_graphs(holder_dir: &Path, zip_name: &str) -> Result<PathBuf> {
    // Zip regex graphs (shared)
    info!(LOG, "Zipping regex graphs");
    let holder_dir_str = holder_dir
        .to_str()
        .ok_or_else(|| anyhow!("holder_dir path must be valid UTF-8"))?;
    run_command(
        "zip",
        &["-r", zip_name, ".", "-i", "*_regex.json"],
        Some(holder_dir_str),
    )
    .await?;

    Ok(holder_dir.join(zip_name))
}

/// Internal helper to derive the public inputs length from the generated
/// Honk verifier contract. We read `NUMBER_OF_PUBLIC_INPUTS` from the given
/// `honk_path` and use that as `public_inputs_length` for the
/// ZKEmailVerifier template.
pub fn derive_public_inputs_length(honk_path: &Path) -> Result<usize> {
    let contents = fs::read_to_string(&honk_path).map_err(|e| {
        anyhow!(
            "Failed to read HonkVerifier at {}: {}",
            honk_path.display(),
            e
        )
    })?;

    let re = Regex::new(r"NUMBER_OF_PUBLIC_INPUTS\s*=\s*(\d+)\s*;")
        .map_err(|e| anyhow!("Failed to compile NUMBER_OF_PUBLIC_INPUTS regex: {}", e))?;

    let caps = re
        .captures(&contents)
        .ok_or_else(|| anyhow!("NUMBER_OF_PUBLIC_INPUTS constant not found in HonkVerifier"))?;

    let value_str = caps
        .get(1)
        .ok_or_else(|| anyhow!("NUMBER_OF_PUBLIC_INPUTS capture group missing"))?
        .as_str();

    let value = value_str.parse::<usize>().map_err(|e| {
        anyhow!(
            "Failed to parse NUMBER_OF_PUBLIC_INPUTS value '{}' as u64: {}",
            value_str,
            e
        )
    })?;

    Ok(value)
}

// build_contracts and contract rendering helpers have been moved to the
// blueprint_pipeline and CLI modules to keep this file focused on generic
// filesystem utilities and shared data structures.
