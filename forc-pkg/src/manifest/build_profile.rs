use serde::{Deserialize, Serialize};
use sway_core::{OptLevel, PrintAsm};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ExperimentalFlags {
    pub new_encoding: bool,
    pub abi_hash_ids: bool,
}

/// Parameters to pass through to the `sway_core::BuildConfig` during compilation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct BuildProfile {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub print_ast: bool,
    pub print_dca_graph: Option<String>,
    pub print_dca_graph_url_format: Option<String>,
    #[serde(default)]
    pub print_ir: bool,
    #[serde(default)]
    pub print_asm: PrintAsm,
    #[serde(default)]
    pub print_bytecode: bool,
    #[serde(default)]
    pub terse: bool,
    #[serde(default)]
    pub time_phases: bool,
    #[serde(default)]
    pub metrics_outfile: Option<String>,
    #[serde(default)]
    pub include_tests: bool,
    #[serde(default)]
    pub json_abi_with_callpaths: bool,
    #[serde(default)]
    pub error_on_warnings: bool,
    #[serde(default)]
    pub reverse_results: bool,
    #[serde(default)]
    pub optimization_level: OptLevel,
    #[serde(default)]
    pub experimental: ExperimentalFlags,
}

impl BuildProfile {
    pub const DEBUG: &'static str = "debug";
    pub const RELEASE: &'static str = "release";
    pub const DEFAULT: &'static str = Self::DEBUG;

    pub fn debug() -> Self {
        Self {
            name: Self::DEBUG.into(),
            print_ast: false,
            print_dca_graph: None,
            print_dca_graph_url_format: None,
            print_ir: false,
            print_asm: PrintAsm::default(),
            print_bytecode: false,
            terse: false,
            time_phases: false,
            metrics_outfile: None,
            include_tests: false,
            json_abi_with_callpaths: false,
            error_on_warnings: false,
            reverse_results: false,
            optimization_level: OptLevel::Opt0,
            experimental: ExperimentalFlags {
                new_encoding: false,
                abi_hash_ids: false,
            },
        }
    }

    pub fn release() -> Self {
        Self {
            name: Self::RELEASE.to_string(),
            print_ast: false,
            print_dca_graph: None,
            print_dca_graph_url_format: None,
            print_ir: false,
            print_asm: PrintAsm::default(),
            print_bytecode: false,
            terse: false,
            time_phases: false,
            metrics_outfile: None,
            include_tests: false,
            json_abi_with_callpaths: false,
            error_on_warnings: false,
            reverse_results: false,
            optimization_level: OptLevel::Opt1,
            experimental: ExperimentalFlags {
                new_encoding: false,
                abi_hash_ids: false,
            },
        }
    }
}

impl Default for BuildProfile {
    fn default() -> Self {
        Self::debug()
    }
}

#[cfg(test)]
mod tests {
    use sway_core::{OptLevel, PrintAsm};

    use crate::{manifest::build_profile::ExperimentalFlags, BuildProfile, PackageManifest};

    #[test]
    fn test_build_profile_custom_release_section() {
        let manifest = PackageManifest::from_dir("./tests/sections").expect("manifest");
        let build_profiles = manifest.build_profile.expect("build profile");
        assert_eq!(build_profiles.len(), 3);

        let expected = BuildProfile::debug();
        let profile = build_profiles.get("debug").expect("debug profile");
        assert_eq!(*profile, expected);

        let expected = BuildProfile {
            name: "".into(),
            print_asm: PrintAsm::r#final(),
            ..BuildProfile::debug()
        };
        let profile = build_profiles.get("custom").expect("custom profile");
        assert_eq!(*profile, expected);

        let expected = BuildProfile {
            name: "".into(),
            print_ast: true,
            print_dca_graph: Some("dca_graph".into()),
            print_dca_graph_url_format: Some("print_dca_graph_url_format".into()),
            print_ir: true,
            print_asm: PrintAsm::all(),
            print_bytecode: true,
            terse: true,
            time_phases: true,
            metrics_outfile: Some("metrics_outfile".into()),
            include_tests: true,
            json_abi_with_callpaths: true,
            error_on_warnings: true,
            reverse_results: true,
            optimization_level: OptLevel::Opt0,
            experimental: ExperimentalFlags {
                new_encoding: true,
                abi_hash_ids: true,
            },
        };
        let profile = build_profiles.get("release").expect("release profile");
        assert_eq!(*profile, expected);
    }
}
