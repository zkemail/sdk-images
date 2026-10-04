use anyhow::{anyhow, Context, Result};
use relayer_utils::LOG;
use slog::info;
use std::process::Stdio;
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, process::Command};

pub async fn run_command(command: &str, args: &[&str], dir: Option<&str>) -> Result<()> {
    let mut cmd = Command::new(command);
    cmd.stdin(Stdio::null()).stdout(Stdio::piped());

    // Set arguments if provided
    if !args.is_empty() {
        cmd.args(args);
    }

    // Set current directory if provided
    if let Some(directory) = dir {
        cmd.current_dir(directory);
    }

    let mut child = cmd.spawn().with_context(|| format!("Failed to spawn command `{command}`"))?;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            info!(LOG, "Command output"; "line" => line);
        }
    }

    let status = child.wait().await?;
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
        .with_context(|| format!("Failed to spawn command `{command}`"))?;

    // Provide input to the command
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(format!("{}\n", input).as_bytes()).await?;
        // Drop stdin to deliver EOF before waiting for commands that read it.
    }

    // Wait for the child process to finish
    info!(LOG, "Waiting for command to finish");
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            info!(LOG, "Command output"; "line" => line);
        }
    }

    let status = child.wait().await?;
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
        .with_context(|| format!("Failed to spawn command `{command}`"))?;

    let mut output_lines = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            info!(LOG, "Command output"; "line" => &line);
            output_lines.push(line);
        }
    }

    let status = child.wait().await?;
    if !status.success() {
        return Err(anyhow!(
            "Command `{}` failed with status: {}",
            command,
            status
        ));
    }

    Ok(output_lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_executable_returns_an_error() {
        let missing = "/definitely-missing-sdk-command";
        assert!(run_command(missing, &[], None).await.is_err());
        assert!(run_command_with_input(missing, &[], None, "test").await.is_err());
        assert!(run_command_and_return_output(missing, &[], None).await.is_err());
    }

    #[tokio::test]
    async fn input_is_closed_and_output_is_preserved() {
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            run_command_with_input("sh", &["-c", "cat"], None, "test"),
        ).await.expect("child must receive EOF").unwrap();
        assert_eq!(run_command_and_return_output("sh", &["-c", "printf 'first\\nsecond\\n'"], None).await.unwrap(), "first\nsecond");
    }
}
