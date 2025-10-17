use std::path::PathBuf;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::AutomationLevel;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalConfig {
    pub weights: PriorityWeights,
    pub attention_threshold: f64,
    pub agent_priority: Vec<String>,
    pub default_editor: String,
    pub qdrant_url: String,
    pub automation_level: AutomationLevel,
    pub dry_run_default: bool,
    pub scan_depth: u8,
    pub watch_interval_secs: u64,
    pub max_projects: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriorityWeights {
    pub needs_human: f64,
    pub risk: f64,
    pub staleness: f64,
    pub impact: f64,
    pub confidence: f64,
}


impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            weights: PriorityWeights::default(),
            attention_threshold: 50.0,
            agent_priority: vec!["claude".to_string(), "cursor".to_string(), "nvim".to_string(), "bash".to_string()],
            default_editor: "nvim".to_string(),
            qdrant_url: "http://localhost:6333".to_string(),
            automation_level: AutomationLevel::L1,
            dry_run_default: true,
            scan_depth: 5,
            watch_interval_secs: 5,
            max_projects: None,
        }
    }
}

impl Default for PriorityWeights {
    fn default() -> Self {
        Self {
            needs_human: 40.0,
            risk: 25.0,
            staleness: 15.0,
            impact: 15.0,
            confidence: 10.0,
        }
    }
}


impl GlobalConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: GlobalConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")?;
        Ok(PathBuf::from(home).join(".config/skm/config.toml"))
    }
    
    pub fn watch_interval(&self) -> Duration {
        Duration::from_secs(self.watch_interval_secs)
    }
}