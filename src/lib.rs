use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Re-export modules
pub mod scanner;
pub mod analyzer;
pub mod reporter;
pub mod rag;
pub mod session;
pub mod autopilot;
pub mod meta;

// Types are already publicly accessible through their definitions below

// Error types
#[derive(Debug, Error)]
pub enum SKMError {
    #[error("Project not found: {path}")]
    ProjectNotFound { path: PathBuf },
    
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
    
    #[error("Qdrant connection failed: {message}")]
    QdrantError { message: String },
    
    #[error("tmux command failed: {command}")]
    TmuxError { command: String },
    
    #[error("File system error: {source}")]
    FsError { #[from] source: std::io::Error },
    
    #[error("Git operation failed: {source}")]
    GitError { #[from] source: git2::Error },
    
    #[error("Serialization error: {source}")]
    SerdeError { #[from] source: serde_json::Error },
    
    #[error("TOML parsing error: {source}")]
    TomlError { #[from] source: toml::de::Error },
}

// Result type alias
pub type Result<T> = std::result::Result<T, SKMError>;

// Core data structures from data-model.md
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: String,
    pub path: PathBuf,
    pub stage: Stage,
    pub next: NextAction,
    pub requires_human: Vec<HumanRequirement>,
    pub priority: f64,
    pub tasks: TaskSummary,
    pub updated: DateTime<Utc>,
    pub git: GitStatus,
    pub project_type: ProjectType,
    pub artifacts: ArtifactStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stage {
    Bootstrap,
    Specify,
    Plan,
    Tasks,
    Implement,
    Test,
    Review,
    Done,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NextAction {
    pub command: String,
    pub description: String,
    pub automated: bool,
    pub risk_level: AutomationLevel,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum HumanRequirement {
    Review,
    Input,
    Fix,
    Test,
    Deploy,
    Decision,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TaskSummary {
    pub total: u32,
    pub completed: u32,
    pub parallel_marked: u32,
    pub blocked: u32,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitStatus {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub clean: bool,
    pub last_commit: Option<DateTime<Utc>>,
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    Generic,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArtifactStatus {
    pub constitution: Option<FileInfo>,
    pub spec: Option<FileInfo>,
    pub plan: Option<FileInfo>,
    pub tasks: Option<FileInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub valid: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AutomationLevel {
    L0,  // Read-only
    L1,  // Low-risk
    L2,  // Medium-risk
    L3,  // High-risk
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PortfolioStatus {
    pub generated_at: DateTime<Utc>,
    pub scan_stats: ScanStats,
    pub projects: Vec<Project>,
    pub summary: StatusSummary,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScanStats {
    pub directories_scanned: u32,
    pub projects_found: u32,
    pub scan_time_ms: u64,
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusSummary {
    pub needs_attention: u32,
    pub total_projects: u32,
    pub by_stage: std::collections::HashMap<Stage, u32>,
    pub total_tasks: u32,
    pub completed_tasks: u32,
    pub avg_priority: f64,
}