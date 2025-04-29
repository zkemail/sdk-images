use anyhow::Result;
use sdk_utils::DecomposedRegex;
use std::fs;
use zk_regex_compiler::{DecomposedRegexConfig, ProvingFramework, RegexPart, gen_from_decomposed};

/// Generates Noir files for the provided decomposed regexes.
pub fn generate_regex_circuits(decomposed_regexes: &Option<Vec<DecomposedRegex>>) -> Result<()> {
    if let Some(decomposed_regexes) = decomposed_regexes {
        for decomposed_regex in decomposed_regexes {
            let mut decomposed_regex_config = Vec::new();
            for part in decomposed_regex.parts.clone() {
                if part.is_public {
                    decomposed_regex_config.push(RegexPart::PublicPattern((
                        part.regex_def,
                        decomposed_regex.max_length,
                    )));
                } else {
                    decomposed_regex_config.push(RegexPart::Pattern(part.regex_def));
                }
            }

            let config = DecomposedRegexConfig {
                parts: decomposed_regex_config,
            };

            let (graph, code) =
                gen_from_decomposed(config, &decomposed_regex.name, ProvingFramework::Noir)?;
            let file_path = format!("./tmp/src/{}_regex.nr", decomposed_regex.name);
            fs::write(file_path, code)?;
            let graph_path = format!("./tmp/{}_regex.json", decomposed_regex.name);
            fs::write(graph_path, serde_json::to_string(&graph)?)?;
        }
    }
    Ok(())
}
