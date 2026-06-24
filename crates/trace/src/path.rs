use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TracePath {
    pub root: String,
    pub path: Vec<String>,
}
