use std::path::Path;
use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::AutomationLevel;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectMetaStore {
    pub version: String,
    pub projects: HashMap<String, ProjectMeta>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectMeta {
    pub impact: Option<u8>,
    pub approved_by_human: bool,
    pub custom_commands: HashMap<String, String>,
    pub agent_command: Option<String>,
    pub automation_level: Option<AutomationLevel>,
    pub auto_approve: Vec<String>,
}

impl Default for ProjectMetaStore {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            projects: HashMap::new(),
        }
    }
}

impl Default for ProjectMeta {
    fn default() -> Self {
        Self {
            impact: None,
            approved_by_human: false,
            custom_commands: HashMap::new(),
            agent_command: None,
            automation_level: None,
            auto_approve: Vec::new(),
        }
    }
}

impl ProjectMetaStore {
    /// Load project metadata from .skm/meta.json
    pub fn load(root: &Path) -> Result<Self> {
        let meta_path = root.join(".skm/meta.json");
        
        if !meta_path.exists() {
            return Ok(Self::default());
        }
        
        let content = fs::read_to_string(&meta_path)?;
        let store: ProjectMetaStore = serde_json::from_str(&content)?;
        Ok(store)
    }
    
    /// Save project metadata to .skm/meta.json
    pub fn save(&self, root: &Path) -> Result<()> {
        let skm_dir = root.join(".skm");
        fs::create_dir_all(&skm_dir)?;
        
        let meta_path = skm_dir.join("meta.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&meta_path, content)?;
        
        Ok(())
    }
    
    /// Get metadata for a specific project
    pub fn get_project(&self, project_id: &str) -> Option<&ProjectMeta> {
        self.projects.get(project_id)
    }
    
    /// Get mutable metadata for a specific project
    pub fn get_project_mut(&mut self, project_id: &str) -> &mut ProjectMeta {
        self.projects.entry(project_id.to_string())
            .or_insert_with(ProjectMeta::default)
    }
    
    /// Set a value for a project
    pub fn set_value(&mut self, project_id: &str, key: &str, value: String) -> Result<()> {
        let meta = self.get_project_mut(project_id);
        
        match key {
            "impact" => {
                meta.impact = Some(value.parse::<u8>()?);
            }
            "approved_by_human" => {
                meta.approved_by_human = value.parse::<bool>()?;
            }
            "agent_command" => {
                meta.agent_command = Some(value);
            }
            _ if key.starts_with("command.") => {
                let cmd_name = key.strip_prefix("command.").unwrap();
                meta.custom_commands.insert(cmd_name.to_string(), value);
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown key: {}", key));
            }
        }
        
        Ok(())
    }
}

/// Cache for portfolio status
#[derive(Serialize, Deserialize, Debug)]
pub struct StatusCache {
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}

impl StatusCache {
    /// Load status cache from .skm/status.json
    pub fn load(root: &Path) -> Result<Option<Self>> {
        let cache_path = root.join(".skm/status.json");
        
        if !cache_path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&cache_path)?;
        let cache: StatusCache = serde_json::from_str(&content)?;
        
        // Check if cache is still fresh (less than 5 minutes old)
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(cache.last_updated);
        
        if age.num_minutes() < 5 {
            Ok(Some(cache))
        } else {
            Ok(None)
        }
    }
    
    /// Save status cache to .skm/status.json
    pub fn save(&self, root: &Path) -> Result<()> {
        let skm_dir = root.join(".skm");
        fs::create_dir_all(&skm_dir)?;
        
        let cache_path = skm_dir.join("status.json");
        let content = serde_json::to_string_pretty(&self)?;
        fs::write(&cache_path, content)?;
        
        Ok(())
    }
}