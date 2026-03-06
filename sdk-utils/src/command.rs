use anyhow::{anyhow, Result};
use relayer_utils::LOG;
use slog::info;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};

pub async fn run_command(command: &str, args: &[&str], dir: Option<&str>) -> Result<()> {
    let mut cmd = Command::new(command);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    // Set arguments if provided
    if !args.is_empty() {
        cmd.args(args);
    }

    // Set current directory if provided
    if let Some(directory) = dir {
        cmd.current_dir(directory);
    }

    let mut child = cmd.spawn().expect("failed to execute process");

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Some(line) = lines.next().transpose()? {
            info!(LOG, "Command output"; "line" => line);
        }
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!(
            "Command `{}` failed with status: {}",
            command,
            status
        ));
    }

    Ok(())
}

/// Run a command with additional environment variables set only for that child
/// process. This avoids mutating global process environment.
pub async fn run_command_with_env(
    command: &str,
    args: &[&str],
    dir: Option<&str>,
    envs: &[(&str, &str)],
) -> Result<()> {
    let mut cmd = Command::new(command);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    if !args.is_empty() {
        cmd.args(args);
    }

    if let Some(directory) = dir {
        cmd.current_dir(directory);
    }

    for (key, value) in envs {
        cmd.env(key, value);
    }

    let mut child = cmd.spawn().expect("failed to execute process");

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Some(line) = lines.next().transpose()? {
            info!(LOG, "Command output"; "line" => line);
        }
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!(
            "Command `{}` failed with status: {}",
            command,
            status
        ));
    }

    Ok(())
}

pub async fn run_command_with_input(
    command: &str,
    args: &[&str],
    dir: Option<&str>,
    input: &str,
) -> Result<()> {
    info!(LOG, "Running command with input"; "command" => command, "args" => format!("{:?}", args));
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .current_dir(dir.unwrap_or("."))
        .spawn()
        .expect("Failed to spawn child process");

    // Provide input to the command
    info!(LOG, "Writing input to command"; "input" => input);
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(format!("{}\n", input).as_bytes())?;
    }

    // Wait for the child process to finish
    info!(LOG, "Waiting for command to finish");
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Some(line) = lines.next().transpose()? {
            info!(LOG, "Command output"; "line" => line);
        }
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!(
            "Command `{}` failed with status: {}",
            command,
            status
        ));
    }

    Ok(())
}

pub async fn run_command_and_return_output(
    command: &str,
    args: &[&str],
    dir: Option<&str>,
) -> Result<String> {
    let mut cmd = Command::new(command);

    // Set arguments if provided
    if !args.is_empty() {
        cmd.args(args);
    }

    // Set current directory if provided
    if let Some(directory) = dir {
        cmd.current_dir(directory);
    }

    // Stream stdout through slog for real-time log visibility.
    // Let stderr inherit so it goes directly to the pod output immediately.
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to execute process");

    let mut output_lines = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line?;
            info!(LOG, "Command output"; "line" => &line);
            output_lines.push(line);
        }
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!(
            "Command `{}` failed with status: {}",
            command,
            status
        ));
    }

    Ok(output_lines.join("\n"))
}

pub async fn run_command_with_env_and_return_output(
    command: &str,
    args: &[&str],
    dir: Option<&str>,
    envs: &[(&str, &str)],
) -> Result<String> {
    let mut cmd = Command::new(command);

    if !args.is_empty() {
        cmd.args(args);
    }

    if let Some(directory) = dir {
        cmd.current_dir(directory);
    }

    for (key, value) in envs {
        cmd.env(key, value);
    }

    let output = cmd.output().expect("failed to execute process");

    if !output.status.success() {
        return Err(anyhow!(
            "Command `{}` failed: {}",
            command,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Like `run_command_with_env` but also accumulates all stdout lines and
/// returns them as a single `String`. Useful when you need both real-time
/// visibility and the captured output for post-processing.
pub async fn run_command_with_env_stream_and_return_output(
    command: &str,
    args: &[&str],
    dir: Option<&str>,
    envs: &[(&str, &str)],
) -> Result<String> {
    let mut cmd = Command::new(command);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    if !args.is_empty() {
        cmd.args(args);
    }

    if let Some(directory) = dir {
        cmd.current_dir(directory);
    }

    for (key, value) in envs {
        cmd.env(key, value);
    }

    let mut child = cmd.spawn().expect("failed to execute process");
    let mut output = String::new();

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line?;
            info!(LOG, "Command output"; "line" => &line);
            output.push_str(&line);
            output.push('\n');
        }
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!(
            "Command `{}` failed with status: {}",
            command,
            status
        ));
    }

    Ok(output)
}
