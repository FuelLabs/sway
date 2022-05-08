use crate::config::{
    comments::Comments, expr::Expressions, fundamentals::Fundamentals, heuristics::Heuristics,
    imports::Imports, items::Items, lists::Lists, literals::Literals, ordering::Ordering,
    user_def::Structures,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SwayFormatManifest {
    pub fundamentals: Fundamentals,
    pub imports: Imports,
    pub ordering: Ordering,
    pub items: Items,
    pub lists: Lists,
    pub literals: Literals,
    pub expressions: Expressions,
    pub heuristics: Heuristics,
    pub structures: Structures,
    pub comments: Comments,
}
