use serde_json::Value;
use swayfmt::{
    config::{
        manifest::Config,
        user_def::{FieldAlignment, Structures},
        whitespace::Whitespace,
    },
    Formatter,
};

const ALIGN_FIELDS_FIELD: &str = "alignFields";
const TAB_SIZE_FIELD: &str = "tabSize";
const TAB_SIZE: u64 = 4;
const ALIGN_FIELDS: bool = true;
const DEFAULT_FIELD_ALIGNMENT: usize = 50;

#[derive(Debug, Clone, Copy)]
pub struct SwayConfig {
    tab_size: u64,
    align_fields: bool,
}

impl SwayConfig {
    pub fn default() -> Self {
        Self {
            align_fields: ALIGN_FIELDS,
            tab_size: TAB_SIZE,
        }
    }

    pub fn with_options(options: Value) -> Self {
        let align_fields = extract_align_fields(&options);
        let tab_size = extract_tab_size(&options);

        Self {
            align_fields,
            tab_size,
        }
    }
}

// note `FormattingOptions` and `SwayConfig` may be similar at this moment,
// but they are not the same thing, `SwayConfig` contains all the data related to the LanguageServer
// while `FormattingOptions` is only the part that is necessary for 'formating'
impl From<SwayConfig> for Formatter {
    fn from(config: SwayConfig) -> Self {
        Self {
            config: Config {
                whitespace: Whitespace {
                    tab_spaces: config.tab_size as usize,
                    ..Default::default()
                },
                structures: Structures {
                    field_alignment: match config.align_fields {
                        true => FieldAlignment::AlignFields(DEFAULT_FIELD_ALIGNMENT),
                        false => FieldAlignment::Off,
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            shape: Default::default(),
        }
    }
}

fn extract_align_fields(options: &Value) -> bool {
    if let Value::Object(options_object) = options {
        if options_object.contains_key(ALIGN_FIELDS_FIELD) {
            if let Value::Bool(value) = options_object.get(ALIGN_FIELDS_FIELD).unwrap() {
                *value
            } else {
                ALIGN_FIELDS
            }
        } else {
            ALIGN_FIELDS
        }
    } else {
        ALIGN_FIELDS
    }
}

fn extract_tab_size(options: &Value) -> u64 {
    if let Value::Object(options_object) = options {
        if options_object.contains_key(TAB_SIZE_FIELD) {
            if let Value::Number(value) = options_object.get(TAB_SIZE_FIELD).unwrap() {
                value.as_u64().unwrap_or(TAB_SIZE)
            } else {
                TAB_SIZE
            }
        } else {
            TAB_SIZE
        }
    } else {
        TAB_SIZE
    }
}
