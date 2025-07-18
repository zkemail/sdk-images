use anyhow::Result;
use regex::Regex;
use sdk_utils::{
    compute_signal_length,
    proto_types::proto_blueprint::{Blueprint, DecomposedRegex},
};
use serde::Serialize;
use std::fs;
use tera::{Context, Tera};
use zk_regex_compiler::{gen_from_decomposed, DecomposedRegexConfig, ProvingFramework, RegexPart};

/// Represents a single decomposed regex, along with computed fields
/// used for generating the circuit template.
#[derive(Serialize)]
struct RegexEntry {
    name: String,
    uppercased_name: String,
    max_match_length: usize,
    regex_circuit_name: String,
    location: String,
    max_length_of_location: usize,
    max_length_of_location_name: String,
    reveal_string: String,
    num_public_parts: usize,
    public_parts_max_length: Vec<usize>,
    is_hashed: bool,
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
    remove_soft_linebreaks: bool,
    regexes: Vec<RegexEntry>,
    external_inputs: Vec<ExternalInputEntry>,
    public_args_string: String,
}

impl From<Blueprint> for CircuitTemplateInputs {
    fn from(value: Blueprint) -> Self {
        let circuit_name = value.circuit_name;
        let email_header_max_length = value.email_header_max_length as usize;
        let email_body_max_length = value.email_body_max_length as usize;
        let ignore_body_hash_check = value.ignore_body_hash_check;
        let enable_header_masking = value.enable_header_masking;
        let enable_body_masking = value.enable_body_masking;
        let remove_soft_linebreaks = value.remove_soft_linebreaks;

        // Process regexes
        let mut regexes = Vec::new();

        for regex in value.decomposed_regexes {
            let name = regex.name.clone();
            let uppercased_name = name.to_uppercase();
            let max_match_length = regex.max_match_length as usize;
            let regex_circuit_name = format!("{}_regex", regex.name);

            // Determine location and its max length
            let (location, max_length_of_location, max_length_of_location_name) =
                if regex.location == "header" {
                    (
                        "emailHeader".to_string(),
                        email_header_max_length as usize,
                        "maxHeaderLength".to_string(),
                    )
                } else if remove_soft_linebreaks {
                    (
                        "decodedEmailBodyIn".to_string(),
                        email_body_max_length as usize,
                        "maxBodyLength".to_string(),
                    )
                } else {
                    (
                        "emailBody".to_string(),
                        email_body_max_length as usize,
                        "maxBodyLength".to_string(),
                    )
                };

            // Compute reveal and indexing strings
            let mut reveal_string = String::new();
            let mut num_public_parts = 0;
            let mut public_parts_max_length = Vec::new();
            let is_hashed = regex.is_hashed.unwrap_or(false);
            let mut regex_idx_name = String::new();
            let mut num_reveal_signals: i32 = -1;
            let mut signal_regex_out_string = String::new();

            for part in &regex.parts {
                if part.is_public == Some(true) {
                    num_reveal_signals += 1;
                    if regex_idx_name.is_empty() {
                        regex_idx_name = format!("{}RegexIdx", name);
                    } else {
                        regex_idx_name
                            .push_str(&format!(", {}RegexIdx{}", name, num_reveal_signals));
                    }

                    num_public_parts += 1;
                    public_parts_max_length.push(part.max_length() as usize);

                    if num_reveal_signals == 0 {
                        signal_regex_out_string.push_str(&format!(
                            ", {}RegexReveal[{}]",
                            name,
                            part.max_length()
                        ));
                    } else {
                        signal_regex_out_string.push_str(&format!(
                            ", {}RegexReveal{}[{}]",
                            name,
                            num_reveal_signals,
                            part.max_length()
                        ));
                    }

                    if reveal_string.is_empty() {
                        reveal_string.push_str(&format!(", {}RegexReveal", name));
                    } else {
                        reveal_string
                            .push_str(&format!(", {}RegexReveal{}", name, num_reveal_signals));
                    }
                }
            }

            // Increment once more to account for indexing
            num_reveal_signals += 1;

            regexes.push(RegexEntry {
                name,
                uppercased_name,
                max_match_length,
                regex_circuit_name,
                location,
                max_length_of_location,
                max_length_of_location_name,
                reveal_string,
                num_public_parts,
                public_parts_max_length,
                is_hashed,
                regex_idx_name,
                num_reveal_signals,
                signal_regex_out_string,
            });
        }

        // Process external inputs
        let external_inputs: Vec<ExternalInputEntry> = value
            .external_inputs
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
            remove_soft_linebreaks,
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
        "remove_soft_linebreaks",
        &circuit_template_input.remove_soft_linebreaks,
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
pub fn generate_regex_circuits(decomposed_regexes: Vec<DecomposedRegex>) -> Result<()> {
    for decomposed_regex in decomposed_regexes {
        let mut decomposed_regex_config = Vec::new();
        for part in decomposed_regex.parts.clone() {
            if part.is_public == Some(true) {
                decomposed_regex_config.push(RegexPart::PublicPattern((
                    part.regex_def,
                    part.max_length.unwrap() as usize,
                )));
            } else {
                decomposed_regex_config.push(RegexPart::Pattern(part.regex_def));
            }
        }

        let config = DecomposedRegexConfig {
            parts: decomposed_regex_config,
        };

        let (graph, code) =
            gen_from_decomposed(config, &decomposed_regex.name, ProvingFramework::Circom)?;

        let file_path = format!("./tmp/regex/{}_regex.circom", decomposed_regex.name);
        fs::write(file_path, code)?;
        let graph_path = format!("./tmp/regex/{}_regex.json", decomposed_regex.name);
        fs::write(graph_path, serde_json::to_string(&graph)?)?;
    }

    Ok(())
}
