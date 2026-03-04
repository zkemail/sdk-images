use anyhow::{anyhow, Result};
use sdk_utils::proto_types::proto_blueprint::DecomposedRegex;
use std::{fs, path::{Path, PathBuf}};
use zk_regex_compiler::{gen_from_decomposed, DecomposedRegexConfig, ProvingFramework, RegexPart};

/// Generates Noir files for the provided decomposed regexes under the given
/// `tmp_dir`. For each decomposed regex it writes:
/// - a shared Noir module AND graph JSON into `<tmp_dir>/regex_graphs`.
/// Returns the shared holder directory (`<tmp_dir>/regex_graphs`).
pub fn generate_regex_circuits(tmp_dir: &Path, decomposed_regexes: &Vec<DecomposedRegex>) -> Result<PathBuf> {
    for decomposed_regex in decomposed_regexes {
        let mut decomposed_regex_config = Vec::new();
        for part in decomposed_regex.parts.clone() {
            if part.is_public == Some(true) {
                let max_length = part.max_length.ok_or_else(|| {
                    anyhow!(
                        "max_length is required for public regex part '{}' in regex '{}', but was not provided",
                        part.regex_def,
                        decomposed_regex.name
                    )
                })? as usize;
                decomposed_regex_config
                    .push(RegexPart::PublicPattern((part.regex_def, max_length)));
            } else {
                decomposed_regex_config.push(RegexPart::Pattern(part.regex_def));
            }
        }

        let config = DecomposedRegexConfig {
            parts: decomposed_regex_config,
        };

        let (graph, code) =
            gen_from_decomposed(config, &decomposed_regex.name, ProvingFramework::Noir)?;

        // Write shared Noir + graph artifacts under `<tmp_dir>/regex_graphs`
        let holder_dir = tmp_dir.join("regex_graphs");
        fs::create_dir_all(&holder_dir)?;

        let shared_nr_path = holder_dir.join(format!("{}_regex.nr", decomposed_regex.name));
        fs::write(&shared_nr_path, &code)?;

        let shared_graph_path = holder_dir.join(format!("{}_regex.json", decomposed_regex.name));
        fs::write(shared_graph_path, serde_json::to_string(&graph)?)?;
    }

    Ok(tmp_dir.join("regex_graphs"))
}
