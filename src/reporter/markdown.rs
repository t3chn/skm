use std::path::Path;
use std::fs;
use anyhow::Result;
use crate::{PortfolioStatus, Stage, HumanRequirement};

/// Generate a markdown report for the portfolio status
pub fn generate_markdown_report(status: &PortfolioStatus) -> String {
    let mut report = String::new();
    
    // Header
    report.push_str(&format!("# SKM Portfolio Status Report\n\n"));
    report.push_str(&format!("Generated: {}\n\n", status.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
    
    // Summary
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- **Total Projects**: {}\n", status.summary.total_projects));
    report.push_str(&format!("- **Need Attention**: {} 🚨\n", status.summary.needs_attention));
    report.push_str(&format!("- **Tasks Progress**: {}/{} completed ({:.0}%)\n", 
        status.summary.completed_tasks,
        status.summary.total_tasks,
        if status.summary.total_tasks > 0 {
            (status.summary.completed_tasks as f64 / status.summary.total_tasks as f64) * 100.0
        } else {
            0.0
        }
    ));
    report.push_str(&format!("- **Average Priority**: {:.1}\n", status.summary.avg_priority));
    report.push_str(&format!("- **Scan Time**: {}ms\n\n", status.scan_stats.scan_time_ms));
    
    // Stage Distribution
    report.push_str("## Stage Distribution\n\n");
    report.push_str("| Stage | Count |\n");
    report.push_str("|-------|-------|\n");
    let stages = [
        Stage::Bootstrap,
        Stage::Specify,
        Stage::Plan,
        Stage::Tasks,
        Stage::Implement,
        Stage::Test,
        Stage::Review,
        Stage::Done,
    ];
    for stage in &stages {
        let count = status.summary.by_stage.get(stage).unwrap_or(&0);
        report.push_str(&format!("| {:?} | {} |\n", stage, count));
    }
    report.push_str("\n");
    
    // Priority Projects (top 10)
    report.push_str("## High Priority Projects\n\n");
    let mut sorted_projects = status.projects.clone();
    sorted_projects.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
    
    if sorted_projects.is_empty() {
        report.push_str("No projects found.\n\n");
    } else {
        report.push_str("| Priority | Project | Stage | Next Action | Human Needed |\n");
        report.push_str("|----------|---------|-------|-------------|---------------|\n");
        
        for (idx, project) in sorted_projects.iter().take(10).enumerate() {
            let human_str = if project.requires_human.is_empty() {
                "No".to_string()
            } else {
                format!("Yes ({})", format_requirements(&project.requires_human))
            };
            
            let priority_emoji = if project.priority > 70.0 {
                "🔴"
            } else if project.priority > 40.0 {
                "🟡"
            } else {
                "🟢"
            };
            
            report.push_str(&format!(
                "| {:.1} {} | {} | {:?} | {} | {} |\n",
                project.priority,
                priority_emoji,
                project.path.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown"),
                project.stage,
                truncate(&project.next.description, 40),
                human_str
            ));
        }
        report.push_str("\n");
    }
    
    // All Projects Details
    report.push_str("## Project Details\n\n");
    for project in &sorted_projects {
        report.push_str(&format!("### {}\n\n", 
            project.path.display()
        ));
        
        report.push_str(&format!("- **Stage**: {:?}\n", project.stage));
        report.push_str(&format!("- **Priority**: {:.1}\n", project.priority));
        report.push_str(&format!("- **Type**: {:?}\n", project.project_type));
        report.push_str(&format!("- **Last Updated**: {}\n", 
            project.updated.format("%Y-%m-%d %H:%M UTC")
        ));
        
        if project.git.is_repo {
            report.push_str(&format!("- **Git Branch**: {}\n", 
                project.git.branch.as_ref().unwrap_or(&"unknown".to_string())
            ));
            report.push_str(&format!("- **Git Status**: {}\n", 
                if project.git.clean { "✅ Clean" } else { "⚠️ Uncommitted changes" }
            ));
        }
        
        report.push_str(&format!("- **Tasks**: {}/{} completed", 
            project.tasks.completed, 
            project.tasks.total
        ));
        if project.tasks.parallel_marked > 0 {
            report.push_str(&format!(" ({} parallel)", project.tasks.parallel_marked));
        }
        if project.tasks.blocked > 0 {
            report.push_str(&format!(" ({} blocked)", project.tasks.blocked));
        }
        report.push_str("\n");
        
        report.push_str(&format!("- **Next Action**: {}\n", project.next.description));
        report.push_str(&format!("  - Command: `{}`\n", project.next.command));
        report.push_str(&format!("  - Automated: {}\n", 
            if project.next.automated { "Yes" } else { "No" }
        ));
        
        if !project.requires_human.is_empty() {
            report.push_str(&format!("- **Requires Human**: {}\n", 
                format_requirements(&project.requires_human)
            ));
        }
        
        report.push_str("\n");
    }
    
    // Errors
    if !status.scan_stats.errors.is_empty() {
        report.push_str("## Errors Encountered\n\n");
        for error in &status.scan_stats.errors {
            report.push_str(&format!("- {}\n", error));
        }
        report.push_str("\n");
    }
    
    // Footer
    report.push_str("---\n");
    report.push_str("*Generated by SKM (Spec-Kit Manager)*\n");
    
    report
}

/// Save the markdown report to a file
pub fn save_markdown_report(status: &PortfolioStatus, path: &Path) -> Result<()> {
    let report = generate_markdown_report(status);
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::write(path, report)?;
    Ok(())
}

fn format_requirements(reqs: &[HumanRequirement]) -> String {
    reqs.iter()
        .map(|r| format!("{:?}", r))
        .collect::<Vec<_>>()
        .join(", ")
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len-3])
    }
}