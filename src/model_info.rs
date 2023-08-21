use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    object: String,
    data: Vec<Model>,
}

#[derive(Debug, Deserialize)]
pub struct Model {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
    permission: Vec<ModelPermission>,
    root: String,
    parent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ModelPermission {
    id: String,
    object: String,
    created: u64,
    allow_create_engine: bool,
    allow_sampling: bool,
    allow_logprobs: bool,
    allow_search_indices: bool,
    allow_view: bool,
    allow_fine_tuning: bool,
    organization: String,
    group: Option<String>,
    is_blocking: bool,
}

// Implement Display for ModelInfo
impl fmt::Display for ModelInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Model Info")?;
        writeln!(f, "  Object: {}", self.object)?;
        for model in &self.data {
            writeln!(f, "  - {}", model)?;
        }
        Ok(())
    }
}

// Implement Display for Model
impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Model")?;
        writeln!(f, "  ID: {}", self.id)?;
        writeln!(f, "  Object: {}", self.object)?;
        writeln!(f, "  Created: {}", self.created)?;
        writeln!(f, "  Owned By: {}", self.owned_by)?;
        writeln!(f, "  Root: {}", self.root)?;
        match &self.parent {
            Some(parent) => writeln!(f, "  Parent: {}", parent)?,
            None => writeln!(f, "  Parent: None")?,
        }

        writeln!(f, "  Permissions:")?;
        for permission in &self.permission {
            writeln!(f, "    - {}", permission)?;
        }

        Ok(())
    }
}

// Implement Display for ModelPermission
impl fmt::Display for ModelPermission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Model Permission")?;
        writeln!(f, "  ID: {}", self.id)?;
        writeln!(f, "  Object: {}", self.object)?;
        writeln!(f, "  Created: {}", self.created)?;
        writeln!(f, "  Allow Create Engine: {}", self.allow_create_engine)?;
        writeln!(f, "  Allow Sampling: {}", self.allow_sampling)?;
        writeln!(f, "  Allow Logprobs: {}", self.allow_logprobs)?;
        writeln!(f, "  Allow Search Indices: {}", self.allow_search_indices)?;
        writeln!(f, "  Allow View: {}", self.allow_view)?;
        writeln!(f, "  Allow Fine Tuning: {}", self.allow_fine_tuning)?;
        writeln!(f, "  Organization: {}", self.organization)?;
        writeln!(
            f,
            "  Group: {}",
            self.group.as_ref().unwrap_or(&"None".to_string())
        )?;
        writeln!(f, "  Is Blocking: {}", self.is_blocking)?;
        Ok(())
    }
}
