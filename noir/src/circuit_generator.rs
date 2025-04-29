use crate::models::CircuitTemplateInputs;
use anyhow::Result;
use regex::Regex;
use tera::{Context, Tera};

/// Generates a Noir circuit file by rendering a Tera template with the provided inputs.
/// After rendering, consecutive newlines are collapsed into a single newline.
pub fn generate_circuit(circuit_template_input: CircuitTemplateInputs) -> Result<String> {
    let mut tera = Tera::default();
    tera.add_template_file("./templates/template.nr.tera", Some("template.nr.tera"))?;

    let mut context: Context = Context::new();
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
        "remove_soft_line_breaks",
        &circuit_template_input.remove_soft_line_breaks,
    );
    context.insert("regexes", &circuit_template_input.regexes);
    context.insert("external_inputs", &circuit_template_input.external_inputs);
    context.insert("output_signals", &circuit_template_input.output_signals);
    context.insert("output_args", &circuit_template_input.output_args);

    let circuit = tera.render("template.nr.tera", &context)?;

    // Collapse multiple newlines into a single newline
    let re = Regex::new(r"\n+")?;
    Ok(re.replace_all(&circuit, "\n").to_string())
}
