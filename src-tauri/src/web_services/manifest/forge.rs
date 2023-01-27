use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ForgeManifest(pub HashMap<String, Vec<String>>);
