use crate::cli::DeployCommand;
use reqwest::{
    blocking::{multipart::Form, Client},
    header::{HeaderMap, AUTHORIZATION},
    StatusCode,
};
use serde_json::{to_string_pretty, value::Value, Map};
use std::fs;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::{error, info};

fn extract_manifest_fields(
    manifest: serde_yaml::Value,
) -> anyhow::Result<(String, String, String, String)> {
    let namespace: String = manifest
        .get(&serde_yaml::Value::String("namespace".into()))
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let identifier: String = manifest
        .get(&serde_yaml::Value::String("identifier".into()))
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let graphql_schema: String = manifest
        .get(&serde_yaml::Value::String("graphql_schema".into()))
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let module: serde_yaml::Value = manifest
        .get(&serde_yaml::Value::String("module".into()))
        .unwrap()
        .to_owned();
    let module_path: String = module
        .get(&serde_yaml::Value::String("wasm".into()))
        .unwrap_or_else(|| {
            module
                .get(&serde_yaml::Value::String("native".into()))
                .unwrap()
        })
        .as_str()
        .unwrap()
        .to_string();

    Ok((namespace, identifier, graphql_schema, module_path))
}

pub fn init(command: DeployCommand) -> anyhow::Result<()> {
    let mut manifest_file = fs::File::open(&command.manifest).unwrap_or_else(|_| {
        panic!(
            "Index manifest file at '{}' does not exist",
            command.manifest.display()
        )
    });
    let mut manifest_contents = String::new();
    manifest_file.read_to_string(&mut manifest_contents)?;
    let manifest: serde_yaml::Value = serde_yaml::from_str(&manifest_contents)?;

    let (namespace, identifier, graphql_schema, module_path) = extract_manifest_fields(manifest)?;

    let mut manifest_buff = Vec::new();
    let mut manifest_reader = BufReader::new(manifest_file);
    manifest_reader.read_to_end(&mut manifest_buff)?;

    let form = Form::new()
        .file("manifest", Path::new(&command.manifest))?
        .file("schema", Path::new(&graphql_schema))?
        .file("wasm", Path::new(&module_path))?;

    let target = format!("{}/api/index/{}/{}", &command.url, &namespace, &identifier);

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        command.auth.unwrap_or_else(|| "fuel".into()).parse()?,
    );

    info!(
        "\n🚀 Deploying index at {} to {}",
        &command.manifest.display(),
        &target
    );

    let res = Client::new()
        .post(&target)
        .multipart(form)
        .headers(headers)
        .send()
        .expect("Failed to deploy index.");

    if res.status() != StatusCode::OK {
        error!(
            "\n❌ {} returned a non-200 response code: {:?}",
            &target,
            res.status()
        );
        return Ok(());
    }

    let res_json = res
        .json::<Map<String, Value>>()
        .expect("Failed to read JSON response.");

    println!("\n{}", to_string_pretty(&res_json)?);

    info!(
        "\n✅ Successfully deployed index at {} to {} \n",
        &command.manifest.display(),
        &target
    );

    Ok(())
}
