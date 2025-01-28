use std::collections::VecDeque;

use anyhow::Result;
use regex::Regex;
use sdk_utils::{compute_signal_length, Blueprint, DecomposedRegex};
use serde::Serialize;
use tera::{Context, Tera};
use zk_regex_compiler::{gen_circom_from_decomposed_regex, DecomposedRegexConfig, RegexPartConfig};

/// Represents a single decomposed regex, along with computed fields
/// used for generating the circuit template.
#[derive(Serialize)]
struct RegexEntry {
    name: String,
    uppercased_name: String,
    max_length: usize,
    regex_circuit_name: String,
    location: String,
    max_length_of_location: usize,
    max_length_of_location_name: String,
    reveal_string: String,
    has_public_parts: bool,
    regex_idx_name: String,
    num_reveal_signals: i32,
    signal_regex_out_string: String,
}

/// Represents an external input to the circuit, along with computed fields.
#[derive(Serialize)]
struct ExternalInputEntry {
    name: String,
    max_length: usize,
    signal_length: usize,
}

/// A struct that holds all the data required to render the circuit template.
#[derive(Serialize)]
pub struct CircuitTemplateInputs {
    circuit_name: String,
    email_header_max_length: usize,
    email_body_max_length: usize,
    ignore_body_hash_check: bool,
    enable_header_masking: bool,
    enable_body_masking: bool,
    remove_soft_line_breaks: bool,
    regexes: Vec<RegexEntry>,
    external_inputs: Vec<ExternalInputEntry>,
    public_args_string: String,
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
            .unwrap_or(!ignore_body_hash_check);

        // Process regexes
        let mut regexes = Vec::new();
        if let Some(decomposed_regexes) = value.decomposed_regexes {
            for regex in decomposed_regexes {
                let name = regex.name.clone();
                let uppercased_name = name.to_uppercase();
                let max_length = regex.max_length as usize;
                let regex_circuit_name = format!("{}Regex", regex.name);

                // Determine location and its max length
                let (location, max_length_of_location, max_length_of_location_name) =
                    if regex.location == "header" {
                        (
                            "emailHeader".to_string(),
                            email_header_max_length,
                            "maxHeaderLength".to_string(),
                        )
                    } else if remove_soft_line_breaks {
                        (
                            "decodedEmailBodyIn".to_string(),
                            email_body_max_length,
                            "maxBodyLength".to_string(),
                        )
                    } else {
                        (
                            "emailBody".to_string(),
                            email_body_max_length,
                            "maxBodyLength".to_string(),
                        )
                    };

                // Compute reveal and indexing strings
                let mut reveal_string = String::new();
                let mut has_public_parts = false;
                let mut regex_idx_name = String::new();
                let mut num_reveal_signals: i32 = -1;
                let mut signal_regex_out_string = String::new();

                for part in &regex.parts {
                    if part.is_public {
                        num_reveal_signals += 1;
                        if regex_idx_name.is_empty() {
                            regex_idx_name = format!("{}RegexIdx", name);
                        } else {
                            regex_idx_name
                                .push_str(&format!(", {}RegexIdx{}", name, num_reveal_signals));
                        }

                        has_public_parts = true;
                        if reveal_string.is_empty() {
                            reveal_string.push_str(&format!(", {}RegexReveal", name));
                        } else {
                            reveal_string
                                .push_str(&format!(", {}RegexReveal{}", name, num_reveal_signals));
                        }
                        if num_reveal_signals == 0 {
                            signal_regex_out_string.push_str(&format!(
                                ", {}RegexReveal[{}]",
                                name, max_length_of_location
                            ));
                        } else {
                            signal_regex_out_string.push_str(&format!(
                                ", {}RegexReveal{}[{}]",
                                name, num_reveal_signals, max_length_of_location
                            ));
                        }
                    }
                }

                // Increment once more to account for indexing
                num_reveal_signals += 1;

                regexes.push(RegexEntry {
                    name,
                    uppercased_name,
                    max_length,
                    regex_circuit_name,
                    location,
                    max_length_of_location,
                    max_length_of_location_name,
                    reveal_string,
                    has_public_parts,
                    regex_idx_name,
                    num_reveal_signals,
                    signal_regex_out_string,
                });
            }
        }

        // Process external inputs
        let external_inputs_data = value.external_inputs.unwrap_or_default();
        let external_inputs: Vec<ExternalInputEntry> = external_inputs_data
            .iter()
            .map(|input| {
                let signal_length = compute_signal_length(input.max_length as usize);
                ExternalInputEntry {
                    name: input.name.clone(),
                    max_length: input.max_length as usize,
                    signal_length,
                }
            })
            .collect();

        // Compute the public args string from external inputs
        let public_args: Vec<String> = external_inputs.iter().map(|i| i.name.clone()).collect();
        let mut public_args_string = public_args.join(", ");
        if !public_args_string.is_empty() {
            public_args_string.insert_str(0, ", ");
        }

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
            public_args_string,
        }
    }
}

/// Generates a CIRCOM circuit file by rendering a Tera template with the provided inputs.
/// After rendering, consecutive newlines are collapsed into a single newline.
pub fn generate_circuit(circuit_template_input: CircuitTemplateInputs) -> Result<String> {
    let mut tera = Tera::default();
    tera.add_template_file("./templates/template.circom.tera", Some("circuit.circom"))?;

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
        "public_args_string",
        &circuit_template_input.public_args_string,
    );

    let circuit = tera.render("circuit.circom", &context)?;

    let re = Regex::new(r"\n+")?;
    Ok(re.replace_all(&circuit, "\n").to_string())
}

/// Generates CIRCOM files for the provided decomposed regexes.
pub fn generate_regex_circuits(decomposed_regexes: Option<Vec<DecomposedRegex>>) -> Result<()> {
    if let Some(decomposed_regexes) = decomposed_regexes {
        for decomposed_regex in decomposed_regexes {
            let mut decomposed_regex_config = VecDeque::new();
            for part in decomposed_regex.parts {
                let part_config = RegexPartConfig {
                    is_public: part.is_public,
                    regex_def: part.regex_def.clone(),
                };
                decomposed_regex_config.push_back(part_config);
            }

            let config = DecomposedRegexConfig {
                parts: decomposed_regex_config,
            };

            gen_circom_from_decomposed_regex(
                &mut config.clone(),
                Some(&format!(
                    "./tmp/regex/{}Regex.circom",
                    decomposed_regex.name
                )),
                Some(&format!("{}Regex", decomposed_regex.name)),
                Some(true),
            )?;
        }
    }
    Ok(())
}
