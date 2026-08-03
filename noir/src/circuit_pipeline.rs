use anyhow::{Result, anyhow};
use relayer_utils::LOG;
use sdk_utils::proto_types::proto_blueprint::Blueprint;
use slog::info;
use std::path::{Path, PathBuf};

use crate::external_command::{
    OracleHash, run_bb_write_solidity_verifier, run_bb_write_vk, run_nargo_compile,
};
use crate::filesystem::{CONTRACT_BUNDLE_FILES, derive_public_inputs_length};
use crate::models::CircuitTemplateInputs;
use crate::template::{ZKEmailVerifierInputs, render_main_nr_circuit, render_zkemail_verifier_sol};

/// Bundles all key filesystem locations and artifacts for a single compiled
/// circuit for a given key size. This is a *compile-only* view: it does not
/// include any zipped artifacts.
#[derive(Debug, Clone)]
pub struct CompiledCircuit {
    /// Key size in bits (e.g. 1024, 2048).
    pub key_size_bits: u32,
    /// Noir project root directory: `<tmp_dir>/<key_size_bits>/circuit`.
    pub circuit_dir: PathBuf,
    /// Contracts root directory: `<tmp_dir>/<key_size_bits>/contracts`.
    pub contracts_dir: PathBuf,
    /// Path to the compiled circuit bytecode JSON.
    pub bytecode_path: PathBuf,
}

/// Internal setup for `build_circuit`. Ensures the `src` directory exists
/// under `circuit_dir`, copies `Nargo.toml` into the root, and copies the
/// shared regex Noir modules into `src`.
fn build_circuit_setup(circuit_dir: &Path, regex_graphs_dir: &Path) -> Result<PathBuf> {
    let src_dir = circuit_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    // Copy Nargo.toml into the Noir project root.
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let nargo_toml_path = manifest_dir.join("Nargo.toml.txt");
    std::fs::copy(&nargo_toml_path, circuit_dir.join("Nargo.toml"))?;

    // Copy shared regex Noir modules into this circuit's src dir.
    if regex_graphs_dir.exists() {
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

    Ok(src_dir)
}

/// Builds a Noir circuit for a given key size in an already-prepared
/// `workdir` by:
/// - calling `build_circuit_setup` to prepare `src`, Nargo, and regex modules
/// - generating `main.nr` and compiling with `nargo`.
///
/// The first parameter, `workdir`, is the working directory for this step.
async fn build_circuit(
    workdir: &Path,
    blueprint: &Blueprint,
    key_size_bits: u32,
    regex_graphs_dir: &Path,
) -> Result<(PathBuf, PathBuf, PathBuf)> {
    info!(LOG, "Generating {}-bit circuit", key_size_bits);

    let src_dir = build_circuit_setup(workdir, regex_graphs_dir)?;

    // Generate the Noir circuit for this key size.

    render_main_nr_circuit(
        CircuitTemplateInputs::from_blueprint_with_key_size(blueprint, key_size_bits),
        &src_dir,
    )?;

    // Compile the Noir circuit
    let (target_dir, bytecode_path) = run_nargo_compile(workdir, "sdk_noir").await?;

    // Derive the verification key
    let vk_path = run_bb_write_vk(workdir, &bytecode_path, &target_dir, OracleHash::Keccak).await?;

    // Generate the Solidity Honk verifier contract
    let solidity_verifier_path =
        run_bb_write_solidity_verifier(workdir, &vk_path, &target_dir.join("HonkVerifier.sol"))
            .await?;

    Ok((workdir.to_path_buf(), bytecode_path, solidity_verifier_path))
}

/// Internal setup for `build_contracts`. Copies the minimal set of contracts
/// template files and the generated Honk verifier into the per-circuit
/// `contracts_dir`. Returns the path to the copied Honk verifier.
fn build_contracts_setup(
    contracts_dir: &Path,
    contracts_root: &Path,
    solidity_verifier_path: &Path,
) -> Result<(PathBuf, PathBuf, PathBuf)> {
    // create all needed subdirs first
    let script_dir = contracts_dir.join("script");
    let src_dir = contracts_dir.join("src");
    let interfaces_dir = src_dir.join("interfaces");

    let hh_scripts_dir = contracts_dir.join("hh-scripts");
    let utils_dir = contracts_dir.join("utils");
    std::fs::create_dir_all(&src_dir)?;
    std::fs::create_dir_all(&script_dir)?;
    std::fs::create_dir_all(&interfaces_dir)?;
    std::fs::create_dir_all(&hh_scripts_dir)?;
    std::fs::create_dir_all(&utils_dir)?;

    for rel in CONTRACT_BUNDLE_FILES {
        let src = contracts_root.join(rel);
        if !src.exists() {
            return Err(anyhow!(
                "Expected contracts file at '{}' to exist",
                src.display()
            ));
        }

        std::fs::copy(&src, &contracts_dir.join(rel))?;
    }

    // Copy the solidity verifier to `contracts/src/HonkVerifier.sol`.
    std::fs::copy(solidity_verifier_path, &src_dir.join("HonkVerifier.sol"))?;

    Ok((script_dir, src_dir, interfaces_dir))
}

/// Validates that a sender domain contains only safe characters (letters,
/// digits, dots, hyphens) to prevent template injection in generated Solidity.
fn validate_sender_domain(domain: &str) -> Result<()> {
    if domain.is_empty() {
        return Err(anyhow!("sender_domain must not be empty"));
    }
    if !domain
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(anyhow!(
            "sender_domain contains invalid characters (only a-z, A-Z, 0-9, '.', '-' are allowed): {:?}",
            domain
        ));
    }
    Ok(())
}

fn build_contracts(
    contracts_dir: &Path,
    blueprint: &Blueprint,
    solidity_verifier_path: &Path,
) -> Result<()> {
    validate_sender_domain(&blueprint.sender_domain)?;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let (_script_dir, src_dir, _interfaces_dir) = build_contracts_setup(
        contracts_dir,
        &manifest_dir.join("contracts"),
        solidity_verifier_path,
    )?;

    render_zkemail_verifier_sol(
        &ZKEmailVerifierInputs {
            sender_domain: blueprint.sender_domain.clone(),
            public_inputs_length: derive_public_inputs_length(solidity_verifier_path)?,
        },
        &src_dir.join("ZKEmailVerifier.sol"),
    )?;

    Ok(())
}

/// Internal setup for `build_circuit_artifacts`. Under the given `key_dir`
/// (e.g. `tmp/1024` or `tmp/2048`), creates the circuit and contracts
/// directories and returns their paths:
/// - `circuit_dir = key_dir/circuit`
/// - `contracts_dir = key_dir/contracts`
fn build_circuit_artifacts_setup(key_dir: &Path) -> Result<(PathBuf, PathBuf)> {
    let circuit_dir = key_dir.join("circuit");
    let contracts_dir = key_dir.join("contracts");

    std::fs::create_dir_all(&circuit_dir)?;
    std::fs::create_dir_all(&contracts_dir)?;

    Ok((circuit_dir, contracts_dir))
}

/// Runs the full per-circuit lifecycle for a single key size:
/// 1. create the circuit and contracts directories under the given `key_dir`
/// 2. build the Noir circuit in the provided circuit dir
/// 3. build the contracts subtree in the provided contracts dir
/// and returns the collected filesystem locations as `CompiledCircuit`
/// (without any zipping).
///
/// The first parameter, `workdir`, is the working directory for this circuit
/// (e.g. typically a key-size dir like `tmp/1024` or `tmp/2048`).
pub async fn build_circuit_artifacts(
    workdir: &Path,
    blueprint: &Blueprint,
    key_size_bits: u32,
    regex_graphs_dir: &Path,
) -> Result<CompiledCircuit> {
    // Setup: create the circuit and contracts directories under this working
    // directory (typically a key-size dir) and obtain their paths.
    let (circuit_dir, contracts_dir) = build_circuit_artifacts_setup(workdir)?;

    // Build the Noir circuit in `circuit_dir`. This function is responsible for
    // creating `src`, copying Nargo, and copying shared regex Noir modules, and
    // returns both the circuit directory and the path to the generated Solidity Honk verifier contract.
    let (circuit_dir, bytecode_path, solidity_verifier_path) =
        build_circuit(&circuit_dir, blueprint, key_size_bits, regex_graphs_dir).await?;

    // After successful compilation (and Honk verifier generation), build the
    // contracts subtree for this circuit in `contracts_dir`, using the
    // generated verifier contract path.
    build_contracts(&contracts_dir, blueprint, &solidity_verifier_path)?;

    Ok(CompiledCircuit {
        key_size_bits,
        circuit_dir,
        contracts_dir,
        bytecode_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::zip_circuit_dir;
    use sdk_utils::run_command_and_return_output;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Verifies that the tmp folder structure (circuit/ + contracts/) and
    /// zip bundling work correctly without compiling a circuit (no nargo, bb).
    #[tokio::test]
    async fn test_folder_structure_and_zip() {
        let test_dir = std::env::temp_dir().join(format!(
            "noir_zip_test_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

        // 1. build_circuit_artifacts_setup: creates circuit/ and contracts/
        let (circuit_dir, contracts_dir) =
            build_circuit_artifacts_setup(&test_dir).unwrap();
        assert!(circuit_dir.exists());
        assert!(contracts_dir.exists());
        assert_eq!(circuit_dir, test_dir.join("circuit"));
        assert_eq!(contracts_dir, test_dir.join("contracts"));

        // 2. build_circuit_setup: copies Nargo.toml and .nr regex modules
        let regex_dir = test_dir.join("regex_graphs");
        std::fs::create_dir_all(&regex_dir).unwrap();
        std::fs::write(regex_dir.join("body_regex.nr"), "// test regex module").unwrap();
        std::fs::write(regex_dir.join("not_a_nr.txt"), "should be skipped").unwrap();

        let src_dir = build_circuit_setup(&circuit_dir, &regex_dir).unwrap();
        assert!(src_dir.join("body_regex.nr").exists(), "should copy .nr files");
        assert!(
            !src_dir.join("not_a_nr.txt").exists(),
            "should skip non-.nr files"
        );
        assert!(
            circuit_dir.join("Nargo.toml").exists(),
            "Nargo.toml should be copied"
        );

        // 3. Fake HonkVerifier.sol with NUMBER_OF_PUBLIC_INPUTS
        let fake_honk = test_dir.join("HonkVerifier.sol");
        std::fs::write(
            &fake_honk,
            "uint256 public constant NUMBER_OF_PUBLIC_INPUTS = 42;\n",
        )
        .unwrap();

        // 4. build_contracts_setup: copies CONTRACT_BUNDLE_FILES + HonkVerifier
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let (_script_dir, contracts_src_dir, _interfaces_dir) =
            build_contracts_setup(&contracts_dir, &manifest_dir.join("contracts"), &fake_honk)
                .unwrap();
        assert!(
            contracts_src_dir.join("HonkVerifier.sol").exists(),
            "HonkVerifier.sol should be copied into contracts/src"
        );
        for rel in CONTRACT_BUNDLE_FILES {
            assert!(
                contracts_dir.join(rel).exists(),
                "contract bundle file '{}' should exist",
                rel
            );
        }

        // 5. render_zkemail_verifier_sol: generates ZKEmailVerifier.sol
        render_zkemail_verifier_sol(
            &ZKEmailVerifierInputs {
                sender_domain: "example.com".to_string(),
                public_inputs_length: derive_public_inputs_length(&fake_honk).unwrap(),
            },
            &contracts_src_dir.join("ZKEmailVerifier.sol"),
        )
        .unwrap();
        assert!(contracts_src_dir.join("ZKEmailVerifier.sol").exists());

        // 6. zip_circuit_dir and verify contents
        let zip_path = test_dir.join("circuit_1024.zip");
        zip_circuit_dir(&test_dir, &zip_path).await.unwrap();
        assert!(zip_path.exists(), "zip file should exist");

        let list_output = run_command_and_return_output(
            "unzip",
            &["-l", zip_path.to_str().unwrap()],
            Some(test_dir.to_str().unwrap()),
        )
        .await
        .unwrap();

        let expected_entries = [
            "circuit/Nargo.toml",
            "circuit/src/",
            "contracts/src/HonkVerifier.sol",
            "contracts/src/ZKEmailVerifier.sol",
        ];
        for entry in &expected_entries {
            assert!(
                list_output.contains(entry),
                "zip should contain '{}'; got:\n{}",
                entry,
                list_output
            );
        }
        for rel in CONTRACT_BUNDLE_FILES {
            let entry = format!("contracts/{}", rel);
            assert!(
                list_output.contains(&entry),
                "zip should contain '{}'; got:\n{}",
                entry,
                list_output
            );
        }

        std::fs::remove_dir_all(&test_dir).ok();
    }
}
