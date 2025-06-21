use std::collections::HashMap;

use clap::{Parser, ValueEnum};

macro_rules! features {
    ($($name:ident = $enabled:literal, $url:literal),* $(,)?) => {
        paste::paste! {
            #[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq, Hash)]
            #[value(rename_all = "snake")]
            pub enum Feature {
                $(
                    [<$name:camel>],
                )*
            }

            impl Feature {
                pub const CFG: &[&str] = &[
                    $(
                        stringify!([<experimental_ $name:snake>]),
                    )*
                ];

                pub fn name(&self) -> &'static str {
                    match self {
                        $(
                            Feature::[<$name:camel>] => {
                                stringify!([<$name:snake>])
                            },
                        )*
                    }
                }

                pub fn url(&self) -> &'static str {
                    match self {
                        $(
                            Feature::[<$name:camel>] => {
                                $url
                            },
                        )*
                    }
                }

                pub fn error_because_is_disabled(&self, span: &sway_types::Span) -> sway_error::error::CompileError {
                    match self {
                        $(
                            Self::[<$name:camel>] => {
                                sway_error::error::CompileError::FeatureIsDisabled {
                                    feature: stringify!([<$name:snake>]).into(),
                                    url: $url.into(),
                                    span: span.clone()
                                }
                            },
                        )*
                    }
                }
            }

            impl std::str::FromStr for Feature {
                type Err = Error;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s {
                        $(
                            stringify!([<$name:snake>]) => {
                                Ok(Self::[<$name:camel>])
                            },
                        )*
                        _ => Err(Error::UnknownFeature(s.to_string())),
                    }
                }
            }

            #[derive(Copy, Clone, Debug, PartialEq, Eq)]
            pub struct ExperimentalFeatures {
                $(
                    pub [<$name:snake>]: bool,
                )*
            }

            impl std::default::Default for ExperimentalFeatures {
                fn default() -> Self {
                    Self {
                        $(
                            [<$name:snake>]: $enabled,
                        )*
                    }
                }
            }

            impl ExperimentalFeatures {
                pub fn set_enabled_by_name(&mut self, feature: &str, enabled: bool) -> Result<(), Error> {
                    let feature = feature.trim();
                    match feature {
                        $(
                            stringify!([<$name:snake>]) => {
                                self.[<$name:snake>] = enabled;
                                Ok(())
                            },
                        )*
                        "" => Ok(()),
                        _ => Err(Error::UnknownFeature(feature.to_string())),
                    }
                }

                pub fn set_enabled(&mut self, feature: Feature, enabled: bool) {
                    match feature {
                        $(
                            Feature::[<$name:camel>] => {
                                self.[<$name:snake>] = enabled
                            },
                        )*
                    }
                }

                /// Used for testing if a `#[cfg(...)]` feature is enabled.
                /// Already prepends "experimental_" to the feature name.
                pub fn is_enabled_for_cfg(&self, cfg: &str) -> Result<bool, Error> {
                    match cfg {
                        $(
                            stringify!([<experimental_ $name:snake>]) => Ok(self.[<$name:snake>]),
                        )*
                        _ => Err(Error::UnknownFeature(cfg.to_string()))
                    }
                }

                $(
                pub fn [<with_ $name:snake>](mut self, enabled: bool) -> Self {
                    self.[<$name:snake>] = enabled;
                    self
                }
                )*
            }
        }
    };
}

impl ExperimentalFeatures {
    /// Experimental features will be applied in the following order:
    /// 1 - manifest (no specific order)
    /// 2 - cli_no_experimental
    /// 3 - cli_experimental
    /// 4 - FORC_NO_EXPERIMENTAL (env var)
    /// 5 - FORC_EXPERIMENTAL (env var)
    pub fn new(
        manifest: &HashMap<String, bool>,
        cli_experimental: &[Feature],
        cli_no_experimental: &[Feature],
    ) -> Result<ExperimentalFeatures, Error> {
        let mut experimental = ExperimentalFeatures::default();

        experimental.parse_from_package_manifest(manifest)?;

        for f in cli_no_experimental {
            experimental.set_enabled(*f, false);
        }

        for f in cli_experimental {
            experimental.set_enabled(*f, true);
        }

        experimental.parse_from_environment_variables()?;

        Ok(experimental)
    }
}

features! {
    new_encoding = true,
    "https://github.com/FuelLabs/sway/issues/5727",
    references = true,
    "https://github.com/FuelLabs/sway/issues/5063",
    const_generics = false,
    "https://github.com/FuelLabs/sway/issues/6860",
}

#[derive(Clone, Debug, Default, Parser)]
pub struct CliFields {
    /// Comma separated list of all experimental features that will be enabled
    #[clap(long, value_delimiter = ',')]
    pub experimental: Vec<Feature>,

    /// Comma separated list of all experimental features that will be disabled
    #[clap(long, value_delimiter = ',')]
    pub no_experimental: Vec<Feature>,
}

#[derive(Debug)]
pub enum Error {
    ParseError(String),
    UnknownFeature(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseError(feature) => f.write_fmt(format_args!(
                "Experimental feature \"{feature}\" cannot be parsed."
            )),
            Error::UnknownFeature(feature) => {
                f.write_fmt(format_args!("Unknown experimental feature: \"{feature}\"."))
            }
        }
    }
}

impl ExperimentalFeatures {
    pub fn parse_from_package_manifest(
        &mut self,
        experimental: &std::collections::HashMap<String, bool>,
    ) -> Result<(), Error> {
        for (feature, enabled) in experimental {
            self.set_enabled_by_name(feature, *enabled)?;
        }
        Ok(())
    }

    /// Enable and disable features using comma separated feature names from
    /// environment variables "FORC_EXPERIMENTAL" and "FORC_NO_EXPERIMENTAL".
    pub fn parse_from_environment_variables(&mut self) -> Result<(), Error> {
        if let Ok(features) = std::env::var("FORC_NO_EXPERIMENTAL") {
            self.parse_comma_separated_list(&features, false)?;
        }

        if let Ok(features) = std::env::var("FORC_EXPERIMENTAL") {
            self.parse_comma_separated_list(&features, true)?;
        }

        Ok(())
    }

    pub fn parse_comma_separated_list(
        &mut self,
        features: impl AsRef<str>,
        enabled: bool,
    ) -> Result<(), Error> {
        for feature in features.as_ref().split(',') {
            self.set_enabled_by_name(feature, enabled)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RollbackEnvVar(String, Option<String>);

    impl RollbackEnvVar {
        pub fn new(name: &str) -> Self {
            let old = std::env::var(name).ok();
            RollbackEnvVar(name.to_string(), old)
        }
    }

    impl Drop for RollbackEnvVar {
        fn drop(&mut self) {
            if let Some(old) = self.1.take() {
                std::env::set_var(&self.0, old);
            }
        }
    }

    #[test]
    fn ok_parse_experimental_features() {
        let _old = RollbackEnvVar::new("FORC_EXPERIMENTAL");
        let _old = RollbackEnvVar::new("FORC_NO_EXPERIMENTAL");

        let mut features = ExperimentalFeatures {
            new_encoding: false,
            ..Default::default()
        };

        std::env::set_var("FORC_EXPERIMENTAL", "new_encoding");
        std::env::set_var("FORC_NO_EXPERIMENTAL", "");
        assert!(!features.new_encoding);
        let _ = features.parse_from_environment_variables();
        assert!(features.new_encoding);

        std::env::set_var("FORC_EXPERIMENTAL", "");
        std::env::set_var("FORC_NO_EXPERIMENTAL", "new_encoding");
        assert!(features.new_encoding);
        let _ = features.parse_from_environment_variables();
        assert!(!features.new_encoding);
    }
}
