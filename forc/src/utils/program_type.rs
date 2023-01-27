#[derive(Debug)]
pub enum ProgramType {
    Contract,
    Script,
    Predicate,
    Library,
}

impl std::fmt::Display for ProgramType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ProgramType::*;
        let s = match self {
            Contract => "contract",
            Script => "script",
            Predicate => "predicate",
            Library => "library",
        };
        write!(f, "{s}")
    }
}
