pub mod deploy;
pub mod run;
pub mod submit;

pub use deploy::Command as Deploy;
pub use run::Command as Run;
pub use submit::Command as Submit;
