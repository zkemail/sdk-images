//! Render the production template into small, isolated end-to-end fixtures.
use anyhow::Result;
use noir::{generate_circuit, CircuitTemplateInputs};
use sdk_utils::proto_types::proto_blueprint::{Blueprint, DecomposedRegex, DecomposedRegexPart};
use std::{fs, path::PathBuf};

fn main() -> Result<()> {
    let root = PathBuf::from(std::env::args().nth(1).expect("fixture output directory"));
    let blueprint = Blueprint {
        circuit_name: "sdk_noir".into(),
        email_header_max_length: 128,
        ignore_body_hash_check: true,
        enable_header_masking: true,
        ..Default::default()
    };
    for bits in [1024, 2048] {
        let dir = root.join(bits.to_string());
        fs::create_dir_all(dir.join("src"))?;
        fs::copy("Nargo.toml.txt", dir.join("Nargo.toml"))?;
        fs::write(dir.join("src/main.nr"), generate_circuit(
            CircuitTemplateInputs::from_blueprint_with_key_size(&blueprint, bits),
        )?)?;
    }
    // Body path: body-hash check, soft line break removal and body masking.
    let body_blueprint = Blueprint {
        email_header_max_length: 256,
        email_body_max_length: 128,
        ignore_body_hash_check: false,
        remove_soft_linebreaks: true,
        enable_body_masking: true,
        ..blueprint.clone()
    };
    let dir = root.join("body");
    fs::create_dir_all(dir.join("src"))?;
    fs::copy("Nargo.toml.txt", dir.join("Nargo.toml"))?;
    fs::write(dir.join("src/main.nr"), generate_circuit(
        CircuitTemplateInputs::from_blueprint_with_key_size(&body_blueprint, 1024),
    )?)?;
    // Compile-only regressions for private regexes and short hashed captures.
    for (name, public) in [("private-regex", false), ("short-hash", true)] {
        let regex = DecomposedRegex {
            name: "check".into(), location: "header".into(), max_match_length: 64,
            is_hashed: Some(public), parts: vec![DecomposedRegexPart {
                regex_def: "[a-z]+".into(), is_public: Some(public), max_length: Some(4),
                ..Default::default()
            }], ..Default::default()
        };
        let mut configured = blueprint.clone();
        configured.decomposed_regexes = vec![regex];
        fs::create_dir_all("tmp/src")?;
        noir::generate_regex_circuits(&configured.decomposed_regexes)?;
        let dir = root.join(name);
        fs::create_dir_all(dir.join("src"))?;
        fs::copy("tmp/src/check_regex.nr", dir.join("src/check_regex.nr"))?;
        fs::copy("Nargo.toml.txt", dir.join("Nargo.toml"))?;
        fs::write(dir.join("src/main.nr"), generate_circuit(
            CircuitTemplateInputs::from_blueprint_with_key_size(&configured, 1024),
        )?)?;
    }
    Ok(())
}
