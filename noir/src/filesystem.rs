use anyhow::{Result, anyhow};
use relayer_utils::LOG;
use sdk_utils::{run_command, upload_to_url};
use slog::info;
use std::{fs, path::Path};

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
/// Each directory gets its own `src` subfolder and `Nargo.toml`.
pub async fn setup_circuit_dir(holder_dir: &Path, subdir: &str) -> Result<()> {
    let base_path = holder_dir.join(subdir);

    // Recreate the base directory
    if base_path.exists() {
        fs::remove_dir_all(&base_path)?;
    }
    fs::create_dir_all(&base_path)?;

    // Create the src directory inside this circuit-specific tmp
    let src_path = base_path.join("src");
    fs::create_dir_all(&src_path)?;

    // Copy Nargo.toml into the circuit-specific tmp
    let nargo_toml_path = Path::new("./Nargo.toml.txt");
    fs::copy(nargo_toml_path, base_path.join("Nargo.toml"))?;

    Ok(())
}

/// Compiles the circuit using nargo in the provided working directory.
pub async fn compile_circuit(cwd: &Path) -> Result<()> {
    let cwd_str = cwd
        .to_str()
        .ok_or_else(|| anyhow!("compile_circuit cwd must be valid UTF-8"))?;
    info!(LOG, "Compiling circuit in {}", cwd_str);
    run_command("nargo", &["compile"], Some(cwd_str)).await?;
    Ok(())
}

/// Zips a single circuit project by archiving its `src` and `Nargo.toml` into
/// `zip_name` placed in the parent directory of `circuit_dir`.
/// Returns the full path to the created zip file.
pub async fn zip_circuit_dir(circuit_dir: &Path, zip_name: &str) -> Result<std::path::PathBuf> {
    let circuit_dir_str = circuit_dir
        .to_str()
        .ok_or_else(|| anyhow!("cleanup_circuit_dir path must be valid UTF-8"))?;

    let parent = circuit_dir
        .parent()
        .ok_or_else(|| anyhow!("circuit_dir must have a parent directory"))?;

    let zip_rel = format!("../{}", zip_name);
    let zip_path = parent.join(zip_name);

    info!(LOG, "Zipping circuit to {}", zip_name);
    run_command(
        "zip",
        &["-r", &zip_rel, "src", "Nargo.toml"],
        Some(circuit_dir_str),
    )
    .await?;

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
