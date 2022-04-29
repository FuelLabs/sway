pub enum Plugin {
    ForcFormat,
    ForcLsp,
    ForcExplore,
    None,
}

pub fn plugin_name_to_enum(n: &str) -> Plugin {
    match n {
        "forc-fmt" => Plugin::ForcFormat,
        "forc-explore" => Plugin::ForcExplore,
        "forc-lsp" => Plugin::ForcLsp,
        _ => Plugin::None,
    }
}

pub fn plugin_description(n: &str) -> String {
    let e = plugin_name_to_enum(n);
    match e {
        Plugin::ForcFormat => "Forc plugin for running the Sway code formatter.".to_string(),
        Plugin::ForcLsp => {
            "Forc plugin for the Sway LSP (Language Server Protocol) implementation".to_string()
        }
        Plugin::ForcExplore => "Forc plugin for running the Fuel Block Explorer.".to_string(),
        Plugin::None => "Unidentified plugin".to_string(),
    }
}
