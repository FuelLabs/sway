use clap::ValueEnum;

macro_rules! features {
    ($($name:ident = $enabled:literal, $url:literal),* $(,)?) => {
        paste::paste! {
            #[derive(Copy, Clone, Debug, ValueEnum)]
            #[value(rename_all = "snake")]
            pub enum Features {
                $(
                    [<$name:camel>],
                )*
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
                pub fn set_enabled(&mut self, feature: &str, enabled: bool) -> Result<(), Error> {
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

                pub fn enable_feature(&mut self, feature: Features, enabled: bool) {
                    match feature {
                        $(
                            Features::[<$name:camel>] => {
                                self.[<$name:snake>] = enabled
                            },
                        )*
                    }
                }

                $(
                pub fn [<with_ $name:snake>](mut self, enabled: bool) -> Self {
                    self.[<$name:snake>] = enabled;
                    self
                }
                )*
            }

            // enum ExperimentalFeatureError {
            //     $(
            //         [<$name:camel Enabled>],
            //         [<$name:camel Disabled>],
            //     )*
            // }

            // impl std::fmt::Display for ExperimentalFeatureError {
            //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            //         let error = match self {
            //             $(
            //                 Self::[<$name:camel Enabled>] => stringify!(Feature [<$name:snake>] needs to be enabled),
            //                 Self::[<$name:camel Disabled>] => stringify!(Feature [<$name:snake>] needs to be disabled),
            //             )*
            //         };
            //         f.write_str(error)
            //     }
            // }
        }
    };
}

features! {
    encoding_v1 = true,
    "https://github.com/FuelLabs/sway/issues/5727",

    storage_domains = false,
    "https://github.com/FuelLabs/sway/pull/6466",
}

#[derive(Debug)]
pub enum Error {
    ParseError(String),
    UnknownFeature(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseError(_) => f.write_str("ParserError"),
            Error::UnknownFeature(feature) => {
                f.write_fmt(format_args!("UnknownFeature: {feature}"))
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
            self.set_enabled(feature, *enabled)?;
        }
        Ok(())
    }

    /// Enable and disable features using comma separated feature names from
    /// environment variables "FORC_EXPERIMENTAL" and "FORC_NO_EXPERIMENTAL".
    pub fn parse_from_environment_variables(&mut self) -> Result<(), Error> {
        if let Ok(features) = std::env::var("FORC_EXPERIMENTAL") {
            self.parse_comma_separated_list(&features, true)?;
        }

        if let Ok(features) = std::env::var("FORC_NO_EXPERIMENTAL") {
            self.parse_comma_separated_list(&features, false)?;
        }

        Ok(())
    }

    pub fn parse_comma_separated_list(
        &mut self,
        features: impl AsRef<str>,
        enabled: bool,
    ) -> Result<(), Error> {
        for feature in features.as_ref().split(',') {
            self.set_enabled(feature, enabled)?;
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

        let mut features = ExperimentalFeatures::default();

        std::env::set_var("FORC_EXPERIMENTAL", "storage_domains");
        std::env::set_var("FORC_NO_EXPERIMENTAL", "");
        assert!(!features.storage_domains);
        let _ = features.parse_from_environment_variables();
        assert!(features.storage_domains);

        std::env::set_var("FORC_EXPERIMENTAL", "");
        std::env::set_var("FORC_NO_EXPERIMENTAL", "storage_domains");
        assert!(features.storage_domains);
        let _ = features.parse_from_environment_variables();
        assert!(!features.storage_domains);
    }
}
