/// We intentionally don't construct this using [serde]'s default deserialization so we get
/// the chance to insert some helpful comments and nicer formatting.
pub(crate) fn default_manifest() -> String {
    let real_name = whoami::realname();

    format!(
        r#"[project]
author  = "{}"
license = "MIT"


[dependencies]
stdlib = {{ path = "../stdlib" }}
"#,
        real_name
    )
}

pub(crate) fn default_program() -> String {
    r#"script {
    fn main() {
        std::println("Hello, world!");
    }
}"#
    .into()
}
