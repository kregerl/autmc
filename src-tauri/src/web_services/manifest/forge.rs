use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ForgeManifest(HashMap<String, Vec<String>>);
