use crate::{build_config::BuildConfig, error::*};

use sway_types::{ident::Ident, span::Span};

use nanoid::nanoid;

pub(crate) fn random_name(span: Span, config: Option<&BuildConfig>) -> Ident {
    let mut name_str: &'static str = Box::leak(nanoid!(32).into_boxed_str());
    if let Some(config) = config {
        while config.generated_names.lock().unwrap().contains(&name_str) {
            name_str = Box::leak(nanoid!(32).into_boxed_str());
        }
        config.generated_names.lock().unwrap().push(name_str);
    }
    Ident::new_with_override(name_str, span)
}
