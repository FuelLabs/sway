use fuel_abi_types::abi::program::{ConcreteTypeId, ProgramABI};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurableDeclarations {
    #[serde(flatten)]
    pub declarations: HashMap<String, ConfigurableDeclaration>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurableDeclaration {
    /// Name of the configurable field.
    pub config_type: String,
    /// Ofset of the configurable field.
    pub offset: u64,
    /// Value of the configurable field.
    pub value: String,
}

impl ConfigurableDeclaration {
    pub fn new(config_type: String, offset: u64, value: String) -> Self {
        Self {
            config_type,
            offset,
            value,
        }
    }
}

impl ConfigurableDeclarations {
    pub fn new(declarations: HashMap<String, ConfigurableDeclaration>) -> Self {
        Self { declarations }
    }

    /// Read `ConfigurableDeclarations` from json file at given `path`.
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let decls = std::fs::read_to_string(path).map_err(|e| {
            anyhow::anyhow!(
                "failed to read to configurable manifest from {}, error: {}",
                path.display(),
                e
            )
        })?;
        let decls: ConfigurableDeclarations = serde_json::from_str(&decls)?;
        Ok(decls)
    }
}

impl TryFrom<ProgramABI> for ConfigurableDeclarations {
    type Error = anyhow::Error;

    fn try_from(value: ProgramABI) -> Result<Self, Self::Error> {
        let concrete_type_lookup: HashMap<&ConcreteTypeId, &str> = value
            .concrete_types
            .iter()
            .map(|conc_type| (&conc_type.concrete_type_id, conc_type.type_field.as_str()))
            .collect();

        let configurables = value
            .configurables
            .unwrap_or_default()
            .iter()
            .map(|configurable| {
                let config_name = configurable.name.as_str();
                let config_concrete_type_id = &configurable.concrete_type_id;
                let config_type_str: &str = concrete_type_lookup
                    .get(config_concrete_type_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!("missing {config_name} type declaration in program abi.")
                    })?;
                let offset = configurable.offset;

                let decl = ConfigurableDeclaration::new(
                    config_type_str.to_string(),
                    offset,
                    "".to_string(),
                );

                Ok((config_name.to_string(), decl))
            })
            .collect::<anyhow::Result<HashMap<_, _>>>()?;

        Ok(Self::new(configurables))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configurable_decl() {
        let decl = r#"{ "configType": "Type A", "value": "Value" }"#;
        let decl_parsed: ConfigurableDeclaration = serde_json::from_str(decl).unwrap();

        assert_eq!(decl_parsed.config_type, "Type A".to_string());
        assert_eq!(decl_parsed.value, "Value".to_string())
    }

    #[test]
    fn test_configurable_decls() {
        let decls = r#"{ "configName": {"configType": "Name", "value": "Value"} }"#;
        let decls_parsed: ConfigurableDeclarations = serde_json::from_str(decls).unwrap();

        assert_eq!(decls_parsed.declarations.len(), 1);

        let decl_parsed = decls_parsed.declarations.iter().nth(0).unwrap();

        assert_eq!(decl_parsed.0, "configName");
        assert_eq!(decl_parsed.1.config_type, "Name");
        assert_eq!(decl_parsed.1.value, "Value");
    }
}
