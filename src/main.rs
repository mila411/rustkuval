use anyhow::{Context, Result};
use async_recursion::async_recursion;
use reqwest::Client;
use serde_json::Value;
use serde_yaml;
use std::fs;
use std::path::Path;
use tokio::task;

const OPENAPI_URL: &str =
    "https://raw.githubusercontent.com/kubernetes/kubernetes/master/api/openapi-spec/swagger.json";
const CACHE_FILE: &str = ".k8s_openapi_cache.json";

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file_or_directory_path>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    let schema = get_openapi_schema().await?;
    let files = get_yaml_files(path)?;

    let tasks: Vec<_> = files
        .into_iter()
        .map(|file| {
            let schema = schema.clone();
            task::spawn(async move { validate_yaml_file(&file, &schema).await })
        })
        .collect();

    for task in tasks {
        if let Ok(result) = task.await {
            if let Err(err) = result {
                eprintln!("{}", err);
            }
        }
    }

    Ok(())
}

async fn get_openapi_schema() -> Result<Value> {
    if Path::new(CACHE_FILE).exists() {
        let data = fs::read_to_string(CACHE_FILE)?;
        let schema: Value = serde_json::from_str(&data)?;
        return Ok(schema);
    }

    let client = Client::new();
    let response = client
        .get(OPENAPI_URL)
        .send()
        .await?
        .json::<Value>()
        .await?;
    let schema_json = serde_json::to_string(&response)?;
    fs::write(CACHE_FILE, schema_json)?;

    Ok(response)
}

fn get_yaml_files(path: &str) -> Result<Vec<String>> {
    let path = Path::new(path);

    if path.is_file() {
        return Ok(vec![path.to_str().unwrap().to_string()]);
    }

    if path.is_dir() {
        let mut files = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                files.push(path.to_str().unwrap().to_string());
            }
        }
        return Ok(files);
    }

    anyhow::bail!("Invalid path: {}", path.display())
}

async fn validate_yaml_file(file: &str, schema: &Value) -> Result<()> {
    let content = fs::read_to_string(file)?;
    let yaml_data: Value = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse YAML in file: {}", file))?;

    // Extract `apiVersion` and `kind`
    let api_version = yaml_data.get("apiVersion").and_then(|v| v.as_str());
    let kind = yaml_data.get("kind").and_then(|v| v.as_str());

    if let (Some(api_version), Some(kind)) = (api_version, kind) {
        println!("File: {}", file);
        println!("  apiVersion: {}", api_version);
        println!("  kind: {}", kind);
    } else {
        eprintln!(
            "File: {}: Missing `apiVersion` or `kind`. Skipping validation.",
            file
        );
        return Ok(());
    }

    let validation_errors = validate_yaml(&yaml_data, schema).await;

    if validation_errors.is_empty() {
        println!("  Validation successful!");
    } else {
        println!("  Validation errors:");
        for err in validation_errors {
            println!("    - {}", err);
        }
    }

    Ok(())
}

#[async_recursion]
async fn validate_yaml(yaml_data: &Value, schema: &Value) -> Vec<String> {
    let mut errors = Vec::new();

    if let Some(required_fields) = schema.get("required").and_then(|v| v.as_array()) {
        for field in required_fields {
            if let Some(field_name) = field.as_str() {
                if !yaml_data.get(field_name).is_some() {
                    errors.push(format!("Missing required field: {}", field_name));
                }
            }
        }
    }

    if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
        for (key, value) in properties {
            if let Some(nested_data) = yaml_data.get(key) {
                let nested_errors = validate_yaml(nested_data, value).await;
                errors.extend(nested_errors);
            }
        }
    }

    errors
}
