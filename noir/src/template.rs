use anyhow::Result;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

use crate::models::CircuitTemplateInputs;

#[derive(Serialize, Clone)]
pub struct MockHonkVerifierInputs {}

#[derive(Serialize, Clone)]
pub struct ZKEmailVerifierInputs {
    pub sender_domain: String,
    pub public_inputs_length: usize,
}

/// Hard-coded templates root relative to the crate.
fn templates_root() -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "templates"].iter().collect()
}

/// Renders the Noir circuit from its Tera template and returns the string.
/// Collapses consecutive newlines into a single newline.
pub fn render_main_nr_circuit(
    circuit_template_input: CircuitTemplateInputs,
    output_dir: &Path,
) -> Result<()> {
    let root = templates_root();
    let mut tera = Tera::default();
    tera.add_template_file(&root.join("main.nr.tera"), Some("main.nr.tera"))?;

    let mut context = Context::new();
    context.insert("circuit_name", &circuit_template_input.circuit_name);
    context.insert(
        "email_header_max_length",
        &circuit_template_input.email_header_max_length,
    );
    context.insert(
        "email_body_max_length",
        &circuit_template_input.email_body_max_length,
    );
    context.insert(
        "ignore_body_hash_check",
        &circuit_template_input.ignore_body_hash_check,
    );
    context.insert(
        "enable_header_masking",
        &circuit_template_input.enable_header_masking,
    );
    context.insert(
        "enable_body_masking",
        &circuit_template_input.enable_body_masking,
    );
    context.insert(
        "remove_soft_linebreaks",
        &circuit_template_input.remove_soft_linebreaks,
    );
    context.insert("regexes", &circuit_template_input.regexes);
    context.insert("external_inputs", &circuit_template_input.external_inputs);
    context.insert("output_signals", &circuit_template_input.output_signals);
    context.insert("output_args", &circuit_template_input.output_args);
    context.insert("key_bits", &circuit_template_input.key_bits);
    context.insert(
        "key_limbs_constant",
        &circuit_template_input.key_limbs_constant,
    );

    let circuit = tera.render("main.nr.tera", &context)?;
    let re = Regex::new(r"\n+")?;
    let circuit = re.replace_all(&circuit, "\n").to_string();

    fs::write(output_dir.join("main.nr"), circuit)?;

    Ok(())
}

/// Renders the mock HonkVerifier Solidity contract from its Tera template
/// and writes it to `output_path`. Used by the CLI for example contracts.
pub fn render_mock_honk_verifier(
    _mock_honk_verifier_inputs: &MockHonkVerifierInputs,
    output_path: &Path,
) -> Result<()> {
    let root = templates_root();
    let mut tera = Tera::default();
    tera.add_template_file(
        &root.join("MockHonkVerifier.sol.tera"),
        Some("HonkVerifier.sol"),
    )?;
    let rendered = tera.render("HonkVerifier.sol", &Context::new())?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, rendered)?;
    Ok(())
}

/// Renders the ZKEmailVerifier Solidity contract from its Tera template using
/// the provided sender domain and public inputs length, and writes it to the
/// given `output_path`.
pub fn render_zkemail_verifier_sol(
    zkemail_verifier_inputs: &ZKEmailVerifierInputs,
    output_path: &Path,
) -> Result<()> {
    let root = templates_root();
    let mut tera = Tera::default();
    tera.add_template_file(
        &root.join("ZKEmailVerifier.sol.tera"),
        Some("ZKEmailVerifier.sol.tera"),
    )?;

    let mut context = Context::new();
    context.insert(
        "public_inputs_length",
        &zkemail_verifier_inputs.public_inputs_length,
    );
    context.insert("sender_domain", &zkemail_verifier_inputs.sender_domain);

    let rendered = tera.render("ZKEmailVerifier.sol.tera", &context)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, rendered)?;

    Ok(())
}
