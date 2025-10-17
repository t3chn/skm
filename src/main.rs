use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::Path;
use std::collections::HashMap;
use chrono::Utc;
use skm::{
    scanner::{finder::ProjectScanner, parser, git},
    analyzer::{stage, priority::{self, PriorityCalculator}},
    meta::{config::GlobalConfig, state::{ProjectMetaStore, StatusCache}},
    Project, PortfolioStatus, ScanStats, StatusSummary, Stage,
};

/// Helper function to check if debug mode is enabled
#[inline]
fn is_debug() -> bool {
    std::env::var("SKM_DEBUG").is_ok()
}

#[derive(Parser)]
#[command(name = "skm")]
#[command(about = "SKM (Spec-Kit Manager) - Intelligent meta-agent for project portfolio management")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan for Spec-Kit projects in current directory
    Scan {
        #[arg(long, default_value = ".")]
        root: String,
        #[arg(long, default_value = "*/.specify")]
        glob: String,
    },
    /// Show status of all projects
    Status {
        #[arg(long, default_value = ".")]
        root: String,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        only: Option<String>,
    },
    /// Generate reports
    Report {
        #[arg(long, default_value = "./.skm/STATUS.md")]
        out: String,
        #[arg(long, default_value = "md")]
        format: String,
    },
    /// Generate digest summaries
    Digest {
        #[arg(long)]
        project: Option<String>,
        mode: String,
        #[arg(long, default_value = "DIGEST.md")]
        out: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Scan { root, glob: _ } => {
            scan_projects(&root).await
        }
        Commands::Status { root, json, only } => {
            show_status(&root, json, only.as_deref()).await
        }
        Commands::Report { out, format } => {
            println!("Generating {} report to {}", format, out);
            // TODO: Implement report functionality
            Ok(())
        }
        Commands::Digest { project, mode, out } => {
            println!("Generating {} digest for {:?} to {}", mode, project, out);
            // TODO: Implement digest functionality
            Ok(())
        }
    }
}

async fn show_status(root_path: &str, json_output: bool, filter: Option<&str>) -> Result<()> {
    let root = Path::new(root_path);
    
    // Try to load cached status first
    if let Ok(Some(cached_status)) = StatusCache::load(root) {
        // StatusCache already checks freshness in load(), so if we got Some, it's fresh
        // Use cached data  
        let portfolio: PortfolioStatus = serde_json::from_value(cached_status.data)?;
        
        // Apply filter if specified
        let mut filtered_portfolio = portfolio.clone();
        if let Some(filter_str) = filter {
            match filter_str {
                "needs-attention" => {
                    let config = GlobalConfig::load()?;
                    filtered_portfolio.projects.retain(|p| p.priority > config.attention_threshold);
                }
                "incomplete" => {
                    filtered_portfolio.projects.retain(|p| p.tasks.completed < p.tasks.total);
                }
                stage if stage.starts_with("stage:") => {
                    let stage_name = &stage[6..];
                    filtered_portfolio.projects.retain(|p| format!("{:?}", p.stage).to_lowercase() == stage_name.to_lowercase());
                }
                _ => {}
            }
        }
        
        if json_output {
            println!("{}", serde_json::to_string_pretty(&filtered_portfolio)?);
        } else {
            display_portfolio_status(&filtered_portfolio);
        }
        
        return Ok(());
    }
    
    // Cache is stale or doesn't exist, rescan
    println!("Cache is stale or missing, rescanning...");
    scan_projects(root_path).await
}

fn display_portfolio_status(portfolio: &PortfolioStatus) {
    println!("=== Portfolio Status ===");
    println!("Generated: {}", portfolio.generated_at.format("%Y-%m-%d %H:%M UTC"));
    println!();
    println!("Total Projects: {}", portfolio.summary.total_projects);
    println!("Need Attention: {}", portfolio.summary.needs_attention);
    println!("Tasks: {}/{} completed ({:.0}%)", 
        portfolio.summary.completed_tasks,
        portfolio.summary.total_tasks,
        if portfolio.summary.total_tasks > 0 {
            (portfolio.summary.completed_tasks as f64 / portfolio.summary.total_tasks as f64) * 100.0
        } else {
            0.0
        }
    );
    println!("Average Priority: {:.1}", portfolio.summary.avg_priority);
    println!();
    
    // Show projects by priority
    let mut projects = portfolio.projects.clone();
    projects.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
    
    println!("Projects (by priority):");
    for project in projects.iter().take(10) {
        let status_icon = if project.tasks.completed == project.tasks.total && project.tasks.total > 0 {
            "✅"
        } else if project.priority > 50.0 {
            "🔴"
        } else if project.priority > 30.0 {
            "🟡"
        } else {
            "🟢"
        };
        
        println!("  {} [{:>5.1}] {} - {:?} - {}/{} tasks", 
            status_icon,
            project.priority,
            project.path.file_name().and_then(|s| s.to_str()).unwrap_or("?"),
            project.stage,
            project.tasks.completed,
            project.tasks.total
        );
    }
    
    if portfolio.projects.len() > 10 {
        println!("  ... and {} more projects", portfolio.projects.len() - 10);
    }
}

async fn scan_projects(root_path: &str) -> Result<()> {
    let root = Path::new(root_path);
    let start_time = std::time::Instant::now();
    
    // Load configuration
    let config = GlobalConfig::load()?;
    let meta_store = ProjectMetaStore::load(root)?;
    
    // Initialize scanner
    let scanner = ProjectScanner::new(root.to_path_buf(), config.scan_depth);
    let projects_found = scanner.find_projects();
    
    // Process each project
    let mut projects = Vec::new();
    let mut errors = Vec::new();
    let mut stage_counts: HashMap<Stage, u32> = HashMap::new();
    let mut total_tasks = 0u32;
    let mut completed_tasks = 0u32;
    
    for project_path in &projects_found {
        match process_project(project_path, &config, &meta_store).await {
            Ok(project) => {
                // Update statistics
                *stage_counts.entry(project.stage.clone()).or_insert(0) += 1;
                total_tasks += project.tasks.total;
                completed_tasks += project.tasks.completed;
                
                // Display project info
                println!("Found: {} [{:?}] Priority: {:.1}", 
                    project.path.display(), 
                    project.stage,
                    project.priority
                );
                
                projects.push(project);
            }
            Err(e) => {
                errors.push(format!("Error processing {}: {}", project_path.display(), e));
            }
        }
    }
    
    // Calculate summary statistics
    let avg_priority = if projects.is_empty() { 
        0.0 
    } else { 
        projects.iter().map(|p| p.priority).sum::<f64>() / projects.len() as f64
    };
    
    let needs_attention = projects.iter()
        .filter(|p| p.priority > config.attention_threshold)
        .count() as u32;
    
    // Create portfolio status
    let portfolio = PortfolioStatus {
        generated_at: Utc::now(),
        scan_stats: ScanStats {
            directories_scanned: projects_found.len() as u32,
            projects_found: projects.len() as u32,
            scan_time_ms: start_time.elapsed().as_millis() as u64,
            errors,
        },
        summary: StatusSummary {
            needs_attention,
            total_projects: projects.len() as u32,
            by_stage: stage_counts,
            total_tasks,
            completed_tasks,
            avg_priority,
        },
        projects,
    };
    
    // Cache the status
    let cache = StatusCache {
        last_updated: Utc::now(),
        data: serde_json::to_value(&portfolio)?,
    };
    cache.save(root)?;
    
    // Save markdown report
    use skm::reporter::save_markdown_report;
    let report_path = root.join(".skm/STATUS.md");
    save_markdown_report(&portfolio, &report_path)?;
    
    // Display summary
    println!("\n=== Scan Complete ===");
    println!("Projects found: {}", portfolio.summary.total_projects);
    println!("Need attention: {}", portfolio.summary.needs_attention);
    println!("Tasks: {}/{} completed", portfolio.summary.completed_tasks, portfolio.summary.total_tasks);
    println!("Average priority: {:.1}", portfolio.summary.avg_priority);
    println!("Scan time: {}ms", portfolio.scan_stats.scan_time_ms);
    
    if !portfolio.scan_stats.errors.is_empty() {
        println!("\nErrors encountered:");
        for error in &portfolio.scan_stats.errors {
            println!("  - {}", error);
        }
    }
    
    Ok(())
}

async fn process_project(
    project_path: &Path, 
    config: &GlobalConfig,
    meta_store: &ProjectMetaStore,
) -> Result<Project> {
    // Parse artifacts from .specify or specs directory
    // Prefer .specify if it has proper artifacts, otherwise check specs
    let specify_path = project_path.join(".specify");
    let specs_path = project_path.join("specs");
    
    if is_debug() {
        eprintln!("[DEBUG] Processing project: {}", project_path.display());
        eprintln!("[DEBUG]   .specify exists: {}", specify_path.exists());
        eprintln!("[DEBUG]   specs exists: {}", specs_path.exists());
    }
    
    // Prefer specs directory if it exists (it usually has the feature directories)
    // Only use .specify if specs doesn't exist or has no content
    let artifacts = if specs_path.exists() {
        let specs_artifacts = parser::parse_artifacts(&specs_path)?;
        // Check if specs has any artifacts
        if specs_artifacts.constitution.is_some() || specs_artifacts.spec.is_some() ||
           specs_artifacts.plan.is_some() || specs_artifacts.tasks.is_some() {
            if is_debug() {
                eprintln!("[DEBUG]   Using artifacts from specs");
            }
            specs_artifacts
        } else if specify_path.exists() {
            // specs has no artifacts, try .specify
            if is_debug() {
                eprintln!("[DEBUG]   specs has no artifacts, trying .specify");
            }
            parser::parse_artifacts(&specify_path)?
        } else {
            specs_artifacts // Use empty artifacts from specs
        }
    } else if specify_path.exists() {
        // No specs, try .specify
        if is_debug() {
            eprintln!("[DEBUG]   Using artifacts from .specify");
        }
        parser::parse_artifacts(&specify_path)?
    } else {
        // Neither exists, return empty artifacts
        if is_debug() {
            eprintln!("[DEBUG]   No artifacts found");
        }
        parser::parse_artifacts(&specify_path)?
    };
    
    // Parse tasks if available
    let tasks = if let Some(ref task_file) = artifacts.tasks {
        parser::parse_tasks_file(&task_file.path)?
    } else {
        Default::default()
    };
    
    // Get git status
    let git_status = git::get_git_status(project_path)?;
    
    // Detect project type first
    use skm::scanner::finder;
    let project_type = finder::detect_project_type(project_path);
    
    // Detect stage
    let current_stage = stage::detect_stage(&artifacts, &project_type);
    
    // Calculate risk and detect requirements
    let has_errors = git::has_recent_errors(project_path)?;
    let risk_level = priority::calculate_risk(&current_stage, &git_status, &tasks, has_errors);
    let human_reqs = priority::detect_human_requirements(&current_stage, &git_status, &tasks);
    
    // Get project metadata
    let project_id = project_path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    let project_meta = meta_store.get_project(&project_id);
    let impact = project_meta.and_then(|m| m.impact).unwrap_or(2);
    let confidence = if project_meta.map(|m| m.approved_by_human).unwrap_or(false) { 2 } else { 1 };
    
    // Calculate priority
    let calculator = PriorityCalculator::new(priority::PriorityWeights {
        needs_human: config.weights.needs_human,
        risk: config.weights.risk,
        staleness: config.weights.staleness,
        impact: config.weights.impact,
        confidence: config.weights.confidence,
    });
    
    let last_updated = artifacts.spec
        .as_ref()
        .map(|f| f.modified)
        .unwrap_or_else(|| Utc::now());
    
    let priority_score = calculator.calculate(
        &human_reqs,
        risk_level,
        last_updated,
        impact,
        confidence,
    );
    
    // Get next action
    let next_action = stage::get_next_action(&current_stage);
    
    Ok(Project {
        id: project_id,
        path: project_path.to_path_buf(),
        stage: current_stage,
        next: next_action,
        requires_human: human_reqs,
        priority: priority_score,
        tasks,
        updated: last_updated,
        git: git_status,
        project_type,
        artifacts,
    })
}
