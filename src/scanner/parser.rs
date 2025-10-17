use std::path::Path;
use std::fs;
use chrono::{DateTime, Utc};
use crate::{Result, FileInfo, ArtifactStatus, TaskSummary};

/// Helper function to check if debug mode is enabled
#[inline]
fn is_debug() -> bool {
    std::env::var("SKM_DEBUG").is_ok()
}

/// Parse Spec-Kit artifacts from a directory
///
/// This function supports two Spec-Kit structures:
/// 1. Direct artifacts: Files (spec.md, plan.md, tasks.md) directly in the directory
/// 2. Feature directories: Numbered subdirectories (001-feature-name, 002-feature-name)
///
/// When multiple feature directories exist, it aggregates the latest artifacts
/// and returns the most recent tasks file.
///
/// # Arguments
/// * `specify_path` - Path to .specify or specs directory
///
/// # Returns
/// * `ArtifactStatus` containing references to found artifacts
pub fn parse_artifacts(specify_path: &Path) -> Result<ArtifactStatus> {
    let status = ArtifactStatus {
        constitution: None,
        spec: None,
        plan: None,
        tasks: None,
    };
    
    // First check if artifacts are directly in the directory
    let direct_artifacts = check_direct_artifacts(specify_path)?;
    if has_any_artifact(&direct_artifacts) {
        return Ok(direct_artifacts);
    }
    
    // If no direct artifacts, check for numbered feature directories (001-feature, 002-feature, etc.)
    // This is the Spec-Kit structure for feature branches
    if let Ok(entries) = fs::read_dir(specify_path) {
        let mut feature_dirs: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter(|e| {
                // Match directories that start with 3 digits (001, 002, etc.)
                e.file_name()
                    .to_str()
                    .map(|name| name.chars().take(3).all(|c| c.is_ascii_digit()))
                    .unwrap_or(false)
            })
            .collect();
        
        if is_debug() && !feature_dirs.is_empty() {
            eprintln!("[DEBUG] Found {} numbered feature directories in {}",
                feature_dirs.len(), specify_path.display());
            for dir in &feature_dirs {
                eprintln!("[DEBUG]   - {}", dir.file_name().to_string_lossy());
            }
        }
        
        // Sort by directory name to get the latest feature
        feature_dirs.sort_by_key(|e| e.file_name());
        
        // Aggregate artifacts from all numbered directories
        // This gives a complete picture of the project's features
        let mut aggregated = ArtifactStatus {
            constitution: None,
            spec: None,
            plan: None,
            tasks: None,
        };
        
        // Look for constitution in parent .specify/memory or specs root
        let memory_constitution = specify_path.parent()
            .map(|p| p.join(".specify/memory/constitution.md"))
            .filter(|p| p.exists());
        if let Some(const_path) = memory_constitution {
            aggregated.constitution = Some(parse_file_info(&const_path)?);
        }
        
        // Get the latest spec, plan, and aggregate all tasks
        let mut all_tasks = Vec::new();
        
        for feature_dir in feature_dirs.iter().rev() {
            let feature_status = check_direct_artifacts(&feature_dir.path())?;
            
            // Use the latest spec if we don't have one
            if aggregated.spec.is_none() && feature_status.spec.is_some() {
                aggregated.spec = feature_status.spec;
            }
            
            // Use the latest plan if we don't have one
            if aggregated.plan.is_none() && feature_status.plan.is_some() {
                aggregated.plan = feature_status.plan;
            }
            
            // Collect all tasks for aggregation
            if let Some(task_file) = feature_status.tasks {
                all_tasks.push(task_file);
            }
        }
        
        // If we found tasks, use the most recent one (could aggregate in future)
        if !all_tasks.is_empty() {
            aggregated.tasks = Some(all_tasks[0].clone());
        }
        
        if has_any_artifact(&aggregated) {
            return Ok(aggregated);
        }
    }
    
    // Return empty status if nothing found
    Ok(status)
}

/// Check for artifacts directly in a directory
fn check_direct_artifacts(path: &Path) -> Result<ArtifactStatus> {
    let mut status = ArtifactStatus {
        constitution: None,
        spec: None,
        plan: None,
        tasks: None,
    };
    
    // Check for constitution.md (can be in .specify/memory/ or directly in specs/)
    let constitution_path = path.join("constitution.md");
    let memory_constitution = path.join("memory/constitution.md");
    if constitution_path.exists() {
        status.constitution = Some(parse_file_info(&constitution_path)?);
    } else if memory_constitution.exists() {
        status.constitution = Some(parse_file_info(&memory_constitution)?);
    }
    
    // Check for spec.md
    let spec_path = path.join("spec.md");
    if spec_path.exists() {
        status.spec = Some(parse_file_info(&spec_path)?);
    }
    
    // Check for plan.md
    let plan_path = path.join("plan.md");
    if plan_path.exists() {
        status.plan = Some(parse_file_info(&plan_path)?);
    }
    
    // Check for tasks.md
    let tasks_path = path.join("tasks.md");
    if tasks_path.exists() {
        status.tasks = Some(parse_file_info(&tasks_path)?);
    }
    
    Ok(status)
}

/// Check if an ArtifactStatus has any artifacts
fn has_any_artifact(status: &ArtifactStatus) -> bool {
    status.constitution.is_some() || 
    status.spec.is_some() || 
    status.plan.is_some() || 
    status.tasks.is_some()
}

fn parse_file_info(path: &Path) -> Result<FileInfo> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    
    Ok(FileInfo {
        path: path.to_path_buf(),
        size: metadata.len(),
        modified: DateTime::<Utc>::from(modified),
        valid: validate_file(path),
    })
}

fn validate_file(path: &Path) -> bool {
    // Basic validation - check if file is readable and not empty
    fs::read_to_string(path)
        .map(|content| !content.trim().is_empty())
        .unwrap_or(false)
}

/// Parse tasks.md file to extract task summary
///
/// Supports multiple task formats:
/// - Checkbox: `- [ ]`, `- [x]`, `- [X]`
/// - Task IDs: `T001:`, `T002:` (standalone)
/// - Emojis: ✅, ❌, 🔄, ⬜
/// - Keywords: `TODO:`, `DONE:`
///
/// Also detects:
/// - Parallel tasks: `[P]`, `(P)`, `||`
/// - Blocked tasks: `[BLOCKED]`, 🚫, ⛔
///
/// # Arguments
/// * `path` - Path to tasks.md file
///
/// # Returns
/// * `TaskSummary` with counts and last activity timestamp
pub fn parse_tasks_file(path: &Path) -> Result<TaskSummary> {
    let content = fs::read_to_string(path)?;

    let mut total = 0;
    let mut completed = 0;
    let mut parallel_marked = 0;
    let mut blocked = 0;

    // Compile regex once outside the loop
    let task_pattern = regex::Regex::new(r"T\d{3,4}:").unwrap();

    if is_debug() {
        eprintln!("[DEBUG] Parsing tasks from: {}", path.display());
    }

    for line in content.lines() {
        let trimmed = line.trim();
        
        // Support multiple task formats:
        // - [ ] task
        // - [x] task  
        // - [X] task
        // - [ ] T001 task (with task ID)
        // - TODO: task
        // - DONE: task
        // - ✅ task
        // - ❌ task
        // - 🔄 task (in progress)
        // T001: task format
        
        // Checkbox format (including those with task IDs like T001)
        if trimmed.starts_with("- [ ]") || trimmed.starts_with("* [ ]") {
            total += 1;
            // Check for parallel and blocked markers
            if line.contains("[P]") || line.contains("(P)") || line.contains("||") {
                parallel_marked += 1;
            }
            if line.contains("[BLOCKED]") || line.contains("🚫") || line.contains("⛔") {
                blocked += 1;
            }
        } else if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") || 
                   trimmed.starts_with("* [x]") || trimmed.starts_with("* [X]") {
            total += 1;
            completed += 1;
            // Also check for parallel markers in completed tasks
            if line.contains("[P]") || line.contains("(P)") || line.contains("||") {
                parallel_marked += 1;
            }
        }
        // Task ID format with colon (T001:, T002:, etc) - standalone format
        else if trimmed.contains(":") && !trimmed.starts_with("- [") && !trimmed.starts_with("* [") {
            // Check if it's a task ID like T001:, T002:, etc (not already counted as checkbox)
            if task_pattern.is_match(trimmed) {
                total += 1;
                // Check if marked as done in various ways
                if line.contains("✅") || line.contains("DONE") || line.contains("[COMPLETE]") ||
                   line.contains("[x]") || line.contains("[X]") {
                    completed += 1;
                }
                if line.contains("[P]") || line.contains("||") {
                    parallel_marked += 1;
                }
                if line.contains("[BLOCKED]") || line.contains("🚫") {
                    blocked += 1;
                }
            }
        }
        // Emoji format
        else if trimmed.starts_with("✅") || trimmed.starts_with("☑") {
            total += 1;
            completed += 1;
        } else if trimmed.starts_with("⬜") || trimmed.starts_with("☐") || 
                  trimmed.starts_with("❌") || trimmed.starts_with("🔄") {
            total += 1;
            if trimmed.starts_with("🔄") {
                // In progress tasks count as incomplete
            }
        }
        // TODO/DONE format
        else if trimmed.starts_with("TODO:") || trimmed.starts_with("- TODO:") {
            total += 1;
        } else if trimmed.starts_with("DONE:") || trimmed.starts_with("- DONE:") {
            total += 1;
            completed += 1;
        }
    }
    
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    
    if is_debug() {
        eprintln!("[DEBUG] Tasks parsed: total={}, completed={}, parallel={}, blocked={}",
            total, completed, parallel_marked, blocked);
    }
    
    Ok(TaskSummary {
        total,
        completed,
        parallel_marked,
        blocked,
        last_activity: Some(DateTime::<Utc>::from(modified)),
    })
}

/// Extract the title from a markdown file (first # heading)
pub fn extract_title(content: &str) -> Option<String> {
    content.lines()
        .find(|line| line.starts_with("# "))
        .map(|line| line.trim_start_matches("# ").trim().to_string())
}

/// Count sections in a markdown file (## headings)
pub fn count_sections(content: &str) -> usize {
    content.lines()
        .filter(|line| line.starts_with("## "))
        .count()
}