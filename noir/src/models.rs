use sdk_utils::{Blueprint, compute_signal_length};
use serde::Serialize;

/// Represents a single decomposed regex, along with computed fields
/// used for generating the circuit template.
#[derive(Serialize)]
pub struct RegexEntry {
    pub name: String,
    pub max_length: usize,
    pub regex_circuit_name: String,
    pub location: String,
    pub max_length_of_location: usize,
    pub num_public_parts: usize,
    pub is_hashed: bool,
    pub hash_packing_size: usize,
    pub hash_inputs: String,
    pub capture_string: String,
}

/// Represents an external input to the circuit, along with computed fields.
#[derive(Serialize)]
pub struct ExternalInputEntry {
    pub name: String,
    pub max_length: usize,
    pub signal_length: usize,
}

/// A struct that holds all the data required to render the circuit template.
#[derive(Serialize)]
pub struct CircuitTemplateInputs {
    pub circuit_name: String,
    pub email_header_max_length: usize,
    pub email_body_max_length: usize,
    pub ignore_body_hash_check: bool,
    pub remove_soft_line_breaks: bool,
    pub regexes: Vec<RegexEntry>,
    pub external_inputs: Vec<ExternalInputEntry>,
    pub output_args: String,
    pub output_signals: String,
}

impl From<Blueprint> for CircuitTemplateInputs {
    fn from(value: Blueprint) -> Self {
        let circuit_name = value.circuit_name.unwrap_or_else(|| "circuit".to_string());
        let email_header_max_length = value.email_header_max_length.unwrap_or(1024) as usize;
        let email_body_max_length = value.email_body_max_length.unwrap_or(2048) as usize;
        let ignore_body_hash_check = value.ignore_body_hash_check.unwrap_or(true);
        let remove_soft_line_breaks = value
            .remove_soft_line_breaks
            .unwrap_or(!ignore_body_hash_check);

        // Process regexes
        let mut regexes = Vec::new();
        if let Some(decomposed_regexes) = value.decomposed_regexes {
            for regex in decomposed_regexes {
                let name = regex.name.clone();
                let max_length = regex.max_length as usize;
                let regex_circuit_name = format!("{}Regex", regex.name);

                // Determine location and its max length
                let (location, max_length_of_location) = if regex.location == "header" {
                    ("header".to_string(), email_header_max_length)
                } else if remove_soft_line_breaks {
                    ("decoded_body".to_string(), email_body_max_length)
                } else {
                    ("body".to_string(), email_body_max_length)
                };

                let mut num_public_parts = 0;
                let is_hashed = regex.is_hashed;
                let mut hash_inputs = Vec::new();
                let mut num_reveal_signals: i32 = -1;
                let mut capture_string = String::new();
                let hash_packing_size = (max_length as f64 / 31.0).ceil() as usize;

                for part in &regex.parts {
                    if part.is_public {
                        num_reveal_signals += 1;

                        for i in 0..hash_packing_size {
                            hash_inputs.push(format!(
                                "{}_capture_{}_packed[{}]",
                                name,
                                num_reveal_signals + 1,
                                i
                            ));
                        }

                        num_public_parts += 1;

                        // Create capture string (e.g., "capture_1, capture_2, ...")
                        if capture_string.is_empty() {
                            capture_string
                                .push_str(&format!("{}_capture_{}", name, num_public_parts));
                        } else {
                            capture_string
                                .push_str(&format!(", {}_capture_{}", name, num_public_parts));
                        }
                    }
                }

                regexes.push(RegexEntry {
                    name,
                    max_length,
                    regex_circuit_name,
                    location,
                    max_length_of_location,
                    num_public_parts,
                    is_hashed: is_hashed.unwrap_or(false),
                    hash_packing_size,
                    hash_inputs: if is_hashed.unwrap_or(false) {
                        hash_inputs.join(", ")
                    } else {
                        String::new()
                    },
                    capture_string,
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

        // Compute output signals and args
        let mut output_signals = String::new();
        let mut output_args = String::new();
        for input in &external_inputs {
            output_signals.push_str(&format!(", {}", input.name));
            output_args.push_str(&format!(", [Field; {}]", input.signal_length));
        }
        for regex in &regexes {
            if regex.num_public_parts > 0 {
                let mut signal = regex.capture_string.clone();
                if regex.is_hashed {
                    signal = signal.replace(",", "_packed,");
                    signal.push_str("_packed");
                }
                output_signals.push_str(&format!(", {}", signal));
                for _ in 0..regex.num_public_parts {
                    if regex.is_hashed {
                        output_args.push_str(&format!(", [Field; {}]", regex.hash_packing_size));
                    } else {
                        output_args.push_str(&format!(", BoundedVec<u8, {}>", regex.max_length));
                    }
                }
            }
        }

        CircuitTemplateInputs {
            circuit_name,
            email_header_max_length,
            email_body_max_length,
            ignore_body_hash_check,
            remove_soft_line_breaks,
            regexes,
            external_inputs,
            output_args,
            output_signals,
        }
    }
}
