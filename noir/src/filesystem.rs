use anyhow::{Result, anyhow};
use relayer_utils::LOG;
use sdk_utils::{proto_types::proto_blueprint::Blueprint, run_command, upload_to_url};
use slog::info;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tera::{Context, Tera};
use regex::Regex;

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

/// Holds key paths under a circuit's `contracts` subtree that callers might
/// want to reference (even if they're not all used immediately).
#[derive(Debug, Clone)]
pub struct ContractsPaths {
    pub root: PathBuf,
    pub honk_verifier: PathBuf,
    pub zkemail_verifier: PathBuf,
}

/// Sets up the temporary directory structure for circuit compilation
pub async fn setup(tmp_dir: &Path) -> Result<()> {
    let tmp_path = tmp_dir;

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

    Ok(())
}

/// Internal helper to set up a circuit-specific tmp directory like `holder_dir/1024` or `holder_dir/2048`.
/// Each directory gets its own Noir project under `noir/` with `src` and `Nargo.toml`.
pub async fn setup_circuit_dir(holder_dir: &Path, subdir: &str) -> Result<()> {
    let base_path = holder_dir.join(subdir);

    // Recreate the base directory
    if base_path.exists() {
        fs::remove_dir_all(&base_path)?;
    }
    fs::create_dir_all(&base_path)?;

    // Create the Noir project root and its src directory (e.g. holder_dir/1024/noir/src)
    let noir_root = base_path.join("noir");
    let src_path = noir_root.join("src");
    fs::create_dir_all(&src_path)?;

    // Copy Nargo.toml into the Noir project root (holder_dir/1024/noir/Nargo.toml)
    let nargo_toml_path = Path::new("./Nargo.toml.txt");
    fs::copy(nargo_toml_path, noir_root.join("Nargo.toml"))?;

    Ok(())
}

/// Compiles the circuit using nargo in the provided working directory.
pub async fn compile_circuit(cwd: &Path) -> Result<()> {
    let cwd_str = cwd
        .to_str()
        .ok_or_else(|| anyhow!("compile_circuit cwd must be valid UTF-8"))?;
    info!(LOG, "Compiling circuit in {}", cwd_str);
    run_command("nargo", &["compile"], Some(cwd_str)).await?;

    // Derive the verification key and Solidity Honk verifier for this circuit.
    // These commands are run in the same working directory so their relative
    // paths (./target/...) resolve correctly.
    info!(LOG, "Writing verification key");
    run_command(
        "bb",
        &[
            "write_vk",
            "--bytecode_path",
            "./target/sdk_noir.json",
            "--output_path",
            "./target",
            "--oracle_hash",
            "keccak",
        ],
        Some(cwd_str),
    )
    .await?;

    info!(LOG, "Writing Solidity Honk verifier");
    run_command(
        "bb",
        &[
            "write_solidity_verifier",
            "--vk_path",
            "./target/vk",
            "--output_path",
            "./target/HonkVerifier.sol",
        ],
        Some(cwd_str),
    )
    .await?;

    Ok(())
}

/// Zips the Noir project at `circuit_dir` (and its sibling `contracts` folder)
/// into `zip_name` under the shared tmp dir and returns the full path to the
/// created zip file.
pub async fn zip_circuit_dir(circuit_dir: &Path, zip_name: &str) -> Result<std::path::PathBuf> {
    // circuit_dir = e.g. tmp/1024/noir (Noir project root)
    let key_dir = circuit_dir
        .parent()
        .ok_or_else(|| anyhow!("circuit_dir must have a parent directory"))?;
    // key_dir = tmp/1024 (directory for this key size; we run zip from here)

    let tmp_dir = key_dir
        .parent()
        .ok_or_else(|| anyhow!("key_dir must have a parent directory"))?;
    // tmp_dir = tmp (zip file is written here)

    let cwd = key_dir
        .to_str()
        .ok_or_else(|| anyhow!("key_dir path must be valid UTF-8"))?;

    let zip_path = tmp_dir.join(zip_name);
    let zip_arg = format!("../{}", zip_name); // relative to cwd (key_dir)

    info!(LOG, "Zipping circuit Noir project (noir + contracts) to {}", zip_name);
    // From within the key-specific directory (e.g. tmp/1024), zip both the Noir
    // project and its sibling contracts directory so the archive contains:
    //   noir/...
    //   contracts/...
    run_command("zip", &["-r", &zip_arg, "noir", "contracts"], Some(cwd)).await?;

    Ok(zip_path)
}

/// Zips regex graphs into `zip_name` under `holder_dir` and returns
/// the full path to the created zip file.
pub async fn zip_regex_graphs(holder_dir: &Path, zip_name: &str) -> Result<std::path::PathBuf> {
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
/// Honk verifier contract. We read `NUMBER_OF_PUBLIC_INPUTS` from
/// `<circuit_dir>/target/HonkVerifier.sol` and use that as
/// `public_inputs_length` for the ZKEmailVerifier template.
fn derive_public_inputs_length(circuit_dir: &Path) -> Result<u64> {
    let honk_path = circuit_dir.join("target").join("HonkVerifier.sol");
    let contents = fs::read_to_string(&honk_path).map_err(|e| {
        anyhow!(
            "Failed to read HonkVerifier at {}: {}",
            honk_path.display(),
            e
        )
    })?;

    let re =
        Regex::new(r"NUMBER_OF_PUBLIC_INPUTS\s*=\s*(\d+)\s*;").map_err(|e| {
            anyhow!("Failed to compile NUMBER_OF_PUBLIC_INPUTS regex: {}", e)
        })?;

    let caps = re
        .captures(&contents)
        .ok_or_else(|| anyhow!("NUMBER_OF_PUBLIC_INPUTS constant not found in HonkVerifier"))?;

    let value_str = caps
        .get(1)
        .ok_or_else(|| anyhow!("NUMBER_OF_PUBLIC_INPUTS capture group missing"))?
        .as_str();

    let value = value_str.parse::<u64>().map_err(|e| {
        anyhow!(
            "Failed to parse NUMBER_OF_PUBLIC_INPUTS value '{}' as u64: {}",
            value_str,
            e
        )
    })?;

    Ok(value)
}

/// Scaffolds a Foundry-compatible contracts package for a single circuit,
/// reading artifacts from `circuit_dir` and writing into `contracts_root`.
///
/// Layout:
/// - contracts/
///   - .env.example
///   - README.md
///   - foundry.toml
///   - package.json
///   - remappings.txt
///   - yarn.lock
///   - src/
///     - interfaces/
///       - IDKIMRegistry.sol
///       - IHonkVerifier.sol
///       - IZKEmailVerifier.sol
///     - HonkVerifier.sol         (copied from `<circuit_dir>/target/HonkVerifier.sol`)
///     - ZKEmailVerifier.sol      (rendered from Tera template)
///   - script/
///     - DeployZKEmailVerifier.s.sol
pub fn scaffold_contracts_for_circuit(
    circuit_dir: &Path,
    contracts_root: &Path,
    blueprint: &Blueprint,
) -> Result<ContractsPaths> {
    let contracts_src = contracts_root.join("src");
    let contracts_interfaces = contracts_src.join("interfaces");
    let contracts_script = contracts_root.join("script");

    // Create directory structure
    fs::create_dir_all(&contracts_interfaces)?;
    fs::create_dir_all(&contracts_script)?;

    // Helper to copy a single file from repo-relative `src` into `dest_dir`.
    fn copy_into(src: &Path, dest_dir: &Path) -> Result<()> {
        let file_name = src
            .file_name()
            .ok_or_else(|| anyhow!("Source path '{}' has no file name", src.display()))?;
        let dest = dest_dir.join(file_name);
        fs::copy(src, &dest).map_err(|e| {
            anyhow!(
                "Failed to copy '{}' to '{}': {}",
                src.display(),
                dest.display(),
                e
            )
        })?;
        Ok(())
    }

    // Base path for the mono-repo contracts package (relative to the Noir crate root).
    let repo_contracts_root = Path::new("./contracts");

    // Copy top-level config/metadata files.
    for name in [
        ".env.example",
        "README.md",
        "foundry.toml",
        "package.json",
        "remappings.txt",
        "yarn.lock",
    ] {
        let src = repo_contracts_root.join(name);
        copy_into(&src, &contracts_root)?;
    }

    // Copy core interfaces.
    let repo_interfaces_root = repo_contracts_root.join("src").join("interfaces");
    for name in ["IDKIMRegistry.sol", "IHonkVerifier.sol", "IZKEmailVerifier.sol"] {
        let src = repo_interfaces_root.join(name);
        copy_into(&src, &contracts_interfaces)?;
    }

    // Copy deploy script.
    let repo_script_root = repo_contracts_root.join("script");
    let deploy_script = repo_script_root.join("DeployZKEmailVerifier.s.sol");
    copy_into(&deploy_script, &contracts_script)?;

    // Copy generated HonkVerifier.sol from the circuit's target dir into contracts/src.
    let honk_source = circuit_dir.join("target").join("HonkVerifier.sol");
    if !honk_source.exists() {
        return Err(anyhow!(
            "Expected HonkVerifier at '{}' but it does not exist",
            honk_source.display()
        ));
    }
    let honk_dest = contracts_src.join("HonkVerifier.sol");
    fs::copy(&honk_source, &honk_dest).map_err(|e| {
        anyhow!(
            "Failed to copy HonkVerifier from '{}' to '{}': {}",
            honk_source.display(),
            honk_dest.display(),
            e
        )
    })?;

    // Compute public_inputs_length from the generated Honk verifier so the
    // Solidity verifier stays in sync with the circuit.
    let public_inputs_length = derive_public_inputs_length(circuit_dir)?;

    // Render ZKEmailVerifier.sol from the Tera template.
    let mut tera = Tera::default();
    tera.add_template_file(
        "./templates/ZKEmailVerifier.sol.tera",
        Some("ZKEmailVerifier.sol.tera"),
    )?;

    let mut context = Context::new();
    context.insert("public_inputs_length", &public_inputs_length);
    context.insert("sender_domain", &blueprint.sender_domain);

    let rendered = tera.render("ZKEmailVerifier.sol.tera", &context)?;
    let zkemail_dest = contracts_src.join("ZKEmailVerifier.sol");
    fs::write(&zkemail_dest, rendered).map_err(|e| {
        anyhow!(
            "Failed to write ZKEmailVerifier to '{}': {}",
            zkemail_dest.display(),
            e
        )
    })?;

    Ok(ContractsPaths {
        root: contracts_root.to_path_buf(),
        honk_verifier: honk_dest,
        zkemail_verifier: zkemail_dest,
    })
}
