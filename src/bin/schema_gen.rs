//! Generates the JSON schema for the gsbt configuration file.

use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let output_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "gsbt.schema.json".to_string());

    let schema = schemars::schema_for!(gsbt::config::Config);
    let mut value = match serde_json::to_value(&schema) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Error converting schema: {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Some(object) = value.as_object_mut() {
        object.insert(
            "$id".to_string(),
            serde_json::json!("https://github.com/devtheops/gsbt/gsbt.schema.json"),
        );
        object.insert("title".to_string(), serde_json::json!("GSBT Configuration"));
        object.insert(
            "description".to_string(),
            serde_json::json!("Configuration schema for the Gameserver Backup Tool (gsbt)"),
        );
    }

    let data = match serde_json::to_string_pretty(&value) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error marshaling schema: {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = fs::write(&output_path, data) {
        eprintln!("Error writing schema file: {err}");
        return ExitCode::FAILURE;
    }

    println!("Schema successfully generated at {output_path}");
    ExitCode::SUCCESS
}
