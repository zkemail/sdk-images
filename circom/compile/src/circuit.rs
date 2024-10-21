use std::process::Command;

use anyhow::{anyhow, Result};

pub async fn compile_circuit(circuit_path: &str) -> Result<()> {
    println!("Unzipping circuit");
    // Unzip circuit files into the artifacts folder
    run_command("unzip", &["-o", circuit_path, "-d", "artifacts"], None).await?;

    println!("Running yarn install");
    // Run yarn install in the artifacts folder
    run_command("yarn", &[], Some("artifacts")).await?;

    println!("Compiling circuit");
    // Compile the circuit
    run_command(
        "circom",
        &["circuit.circom", "--c", "-l", "./node_modules"],
        Some("artifacts"),
    )
    .await?;

    println!("Running make");
    // Run make in the circuit_cpp folder
    run_command("make", &[], Some("artifacts/circuit_cpp")).await?;

    println!("Creating zip file");
    // Create a zip file with the compiled circuit
    run_command(
        "zip",
        &["-r", "compiled_circuit.zip", "circuit_cpp"],
        Some("artifacts"),
    )
    .await?;

    Ok(())
}

async fn run_command(command: &str, args: &[&str], dir: Option<&str>) -> Result<()> {
    let mut cmd = Command::new(command);

    // Set arguments if provided
    if !args.is_empty() {
        cmd.args(args);
    }

    // Set current directory if provided
    if let Some(directory) = dir {
        cmd.current_dir(directory);
    }

    let output = cmd.output().expect("failed to execute process");

    if !output.status.success() {
        return Err(anyhow!(
            "Command `{}` failed: {}",
            command,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}
