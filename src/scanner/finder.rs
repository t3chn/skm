use std::path::{Path, PathBuf};
use walkdir::{WalkDir, DirEntry};
use crate::ProjectType;

pub struct ProjectScanner {
    root: PathBuf,
    max_depth: usize,
    glob_pattern: String,
}

impl ProjectScanner {
    pub fn new(root: PathBuf, max_depth: u8) -> Self {
        Self {
            root,
            max_depth: max_depth as usize,
            glob_pattern: "*/{.specify,specs}".to_string(),
        }
    }
    
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }
    
    pub fn with_pattern(mut self, pattern: String) -> Self {
        self.glob_pattern = pattern;
        self
    }
    
    /// Find projects with .specify or specs directories
    pub fn find_projects(&self) -> Vec<PathBuf> {
        let mut projects = Vec::new();
        let mut seen_projects = std::collections::HashSet::new();
        
        for entry in WalkDir::new(&self.root)
            .max_depth(self.max_depth)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if self.is_specify_dir(&entry) {
                if let Some(project_path) = entry.path().parent() {
                    // Skip if this is inside another project's .specify directory
                    // (e.g., skip /project/.specify/specs if we already have /project)
                    let path_str = project_path.to_string_lossy();
                    if path_str.contains("/.specify/") {
                        continue;
                    }
                    
                    // Only add if we haven't seen this project yet
                    if seen_projects.insert(project_path.to_path_buf()) {
                        projects.push(project_path.to_path_buf());
                    }
                }
            }
        }
        
        projects
    }
    
    fn is_specify_dir(&self, entry: &DirEntry) -> bool {
        if !entry.file_type().is_dir() {
            return false;
        }
        
        let name = entry.file_name();
        if name == ".specify" || name == "specs" {
            // For specs directories, ensure they're not inside .specify
            if name == "specs" {
                let path_str = entry.path().to_string_lossy();
                // Skip specs that are inside .specify directories
                return !path_str.contains("/.specify/");
            }
            true
        } else {
            false
        }
    }
}

/// Detect project type based on language-specific files
pub fn detect_project_type(path: &Path) -> ProjectType {
    // Check for Rust project
    if path.join("Cargo.toml").exists() {
        return ProjectType::Rust;
    }
    
    // Check for Node.js project
    if path.join("package.json").exists() {
        return ProjectType::Node;
    }
    
    // Check for Python project
    if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
        return ProjectType::Python;
    }
    
    // Check for Go project
    if path.join("go.mod").exists() {
        return ProjectType::Go;
    }
    
    // Check for generic source directories
    if path.join("src").exists() || path.join("lib").exists() {
        return ProjectType::Generic;
    }
    
    ProjectType::Unknown
}

/// Check if a directory should be ignored (e.g., node_modules, target)
pub fn should_ignore(path: &Path) -> bool {
    let ignore_dirs = ["node_modules", "target", ".git", "dist", "build", "__pycache__"];
    
    if let Some(file_name) = path.file_name() {
        if let Some(name) = file_name.to_str() {
            return ignore_dirs.contains(&name);
        }
    }
    
    false
}