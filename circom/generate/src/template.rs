use std::collections::VecDeque;

use anyhow::Result;
use serde::Serialize;
use tera::{Context, Tera};
use zk_regex_compiler::{gen_circom_from_decomposed_regex, DecomposedRegexConfig, RegexPartConfig};

use crate::blueprint::{Blueprint, DecomposedRegex};

#[derive(Serialize)]
struct Regex {
    name: String,
    uppercased_name: String, // Computed field
    max_length: usize,
    regex_circuit_name: String, // Computed field
    location: String,
    max_length_of_location: usize,
}

#[derive(Serialize)]
struct ExternalInput {
    name: String,
    max_length: usize,
    signal_length: usize, // Computed field
}

#[derive(Serialize)]
pub struct CircuitTemplateInputs {
    circuit_name: String,
    email_header_max_length: usize,
    email_body_max_length: usize,
    ignore_body_hash_check: bool,
    enable_header_masking: bool,
    enable_body_masking: bool,
    remove_soft_line_breaks: bool,
    regexes: Vec<Regex>,
    external_inputs: Vec<ExternalInput>,
    public_args_length: usize,  // Computed field
    public_args_string: String, // Computed field
}

impl From<Blueprint> for CircuitTemplateInputs {
    fn from(value: Blueprint) -> Self {
        let circuit_name = value.circuit_name.unwrap_or_else(|| "circuit".to_string());
        let email_header_max_length = value.email_header_max_length.unwrap_or(1024) as usize;
        let email_body_max_length = value.email_body_max_length.unwrap_or(2048) as usize;
        let ignore_body_hash_check = value.ignore_body_hash_check.unwrap_or(false);
        let enable_header_masking = value.enable_header_masking.unwrap_or(false);
        let enable_body_masking = value.enable_body_masking.unwrap_or(false);
        let remove_soft_line_breaks = value
            .remove_soft_line_breaks
            .unwrap_or(if !ignore_body_hash_check { true } else { false });

        let mut regexes = Vec::new();
        for regex in value.decomposed_regexes.iter() {
            let name = regex.name.to_string();
            let uppercased_name = name.to_uppercase();
            let max_length = regex.max_length as usize;
            let regex_circuit_name = format!("{}Regex", regex.name);
            let location = if regex.location == "header" {
                "Header".to_string()
            } else {
                "Body".to_string()
            };
            let max_length_of_location = if location == "Header" {
                email_header_max_length
            } else {
                email_body_max_length
            };
            regexes.push(Regex {
                name,
                uppercased_name,
                max_length,
                regex_circuit_name,
                location,
                max_length_of_location,
            });
        }

        let mut external_inputs = Vec::new();
        for input in value.external_inputs.unwrap_or_default().iter() {
            let name = input.name.to_string();
            let max_length = input.max_length as usize;
            let signal_length = compute_signal_length(max_length);
            external_inputs.push(ExternalInput {
                name,
                max_length,
                signal_length,
            });
        }

        let public_args: Vec<String> = external_inputs.iter().map(|i| i.name.clone()).collect();
        let public_args_length = public_args.len();
        let public_args_string = public_args.join(", ");

        CircuitTemplateInputs {
            circuit_name,
            email_header_max_length,
            email_body_max_length,
            ignore_body_hash_check,
            enable_header_masking,
            enable_body_masking,
            remove_soft_line_breaks,
            regexes,
            external_inputs,
            public_args_length,
            public_args_string,
        }
    }
}

fn compute_signal_length(max_length: usize) -> usize {
    (max_length / 31) + if max_length % 31 != 0 { 1 } else { 0 }
}

pub fn generate_circuit(circuit_template_input: CircuitTemplateInputs) -> Result<String> {
    // Initialize Tera and add the template
    let mut tera = Tera::default();
    tera.add_template_file("./template/template.circom.tera", Some("circuit.circom"))?;

    // Create the context and populate it
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
        "remove_soft_line_breaks",
        &circuit_template_input.remove_soft_line_breaks,
    );
    context.insert("regexes", &circuit_template_input.regexes);
    context.insert("external_inputs", &circuit_template_input.external_inputs);
    context.insert(
        "public_args_length",
        &circuit_template_input.public_args_length,
    );
    context.insert(
        "public_args_string",
        &circuit_template_input.public_args_string,
    );

    Ok(tera.render("circuit.circom", &context)?)
}

pub fn generate_regex_circuits(decomposed_regexes: Vec<DecomposedRegex>) -> Result<()> {
    for decomposed_regex in decomposed_regexes.iter() {
        let mut decomposed_regex_config = VecDeque::new();
        for part in decomposed_regex.parts.iter() {
            let part_config = RegexPartConfig {
                is_public: part.is_public,
                regex_def: part.regex_def.clone(),
            };
            decomposed_regex_config.push_back(part_config);
        }
        let mut config = DecomposedRegexConfig {
            parts: decomposed_regex_config,
        };

        gen_circom_from_decomposed_regex(
            &mut config,
            Some(&format!("./regex/{}Regex.circom", decomposed_regex.name)),
            Some(&format!("{}Regex", decomposed_regex.name)),
            Some(true),
        )?;
    }
    Ok(())
}
