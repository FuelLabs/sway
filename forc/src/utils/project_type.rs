use anyhow::Result;

#[derive(Debug)]
pub enum ProjectType {
    Contract,
    Predicate,
    Script,
    Library,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ProjectType::*;
        let s = match self {
            Contract => "contract",
            Predicate => "predicate",
            Library => "library",
            Script => "script",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for ProjectType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        use ProjectType::*;
        Ok(match s.to_lowercase().as_str() {
            "contract" => Contract,
            "predicate" => Predicate,
            "script" => Script,
            "library" => Library,
            otherwise => anyhow::bail!(
                "Unrecognized project type \"{}\": \
            \n Possible Types:\n - contract\n - script\n - library\n - predicate",
                otherwise
            ),
        })
    }
}
