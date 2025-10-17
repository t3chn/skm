use chrono::{Utc, DateTime};
use crate::{Stage, HumanRequirement, GitStatus, TaskSummary};

pub struct PriorityCalculator {
    pub weights: PriorityWeights,
}

#[derive(Debug, Clone)]
pub struct PriorityWeights {
    pub needs_human: f64,
    pub risk: f64,
    pub staleness: f64,
    pub impact: f64,
    pub confidence: f64,
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

impl PriorityCalculator {
    pub fn new(weights: PriorityWeights) -> Self {
        Self { weights }
    }
    
    /// Calculate priority score for a project
    /// Formula: w1*NeedsHuman + w2*Risk + w3*Staleness + w4*Impact - w5*Confidence
    pub fn calculate(
        &self,
        requires_human: &[HumanRequirement],
        risk_level: u8,
        last_updated: DateTime<Utc>,
        impact: u8,
        confidence: u8,
    ) -> f64 {
        let needs_human = if requires_human.is_empty() { 0.0 } else { 1.0 };
        let risk = normalize_risk(risk_level);
        let staleness = calculate_staleness(last_updated);
        let impact_norm = normalize_impact(impact);
        let confidence_norm = normalize_confidence(confidence);
        
        self.weights.needs_human * needs_human
            + self.weights.risk * risk
            + self.weights.staleness * staleness
            + self.weights.impact * impact_norm
            - self.weights.confidence * confidence_norm
    }
}

/// Calculate risk level (0-3) based on various factors
pub fn calculate_risk(
    stage: &Stage,
    git_status: &GitStatus,
    tasks: &TaskSummary,
    has_errors: bool,
) -> u8 {
    let mut risk = 0;
    
    // Add risk for build/test errors
    if has_errors {
        risk += 1;
    }
    
    // Add risk for many parallel branches
    if tasks.parallel_marked > 3 {
        risk += 1;
    }
    
    // Add risk for blocked tasks
    if tasks.blocked > 0 {
        risk += 1;
    }
    
    // Add risk for uncommitted changes
    if !git_status.clean {
        risk += 1;
    }
    
    // Cap at 3
    risk.min(3)
}

/// Normalize risk to 0-1 range
fn normalize_risk(risk: u8) -> f64 {
    (risk as f64) / 3.0
}

/// Calculate staleness based on days since last update (normalized to 0-1, max 7 days)
fn calculate_staleness(last_updated: DateTime<Utc>) -> f64 {
    let now = Utc::now();
    let duration = now.signed_duration_since(last_updated);
    let days = duration.num_days() as f64;
    
    // Normalize to 0-1 range, max at 7 days
    (days / 7.0).min(1.0).max(0.0)
}

/// Normalize impact (1-3) to 0-1 range
fn normalize_impact(impact: u8) -> f64 {
    match impact {
        1 => 0.33,
        2 => 0.66,
        3 => 1.0,
        _ => 0.5, // Default middle value
    }
}

/// Normalize confidence (0-2) to 0-1 range
fn normalize_confidence(confidence: u8) -> f64 {
    (confidence as f64) / 2.0
}

/// Determine which human requirements are needed based on stage
pub fn detect_human_requirements(
    stage: &Stage,
    git_status: &GitStatus,
    tasks: &TaskSummary,
) -> Vec<HumanRequirement> {
    let mut requirements = Vec::new();
    
    match stage {
        Stage::Bootstrap | Stage::Specify | Stage::Plan => {
            requirements.push(HumanRequirement::Input);
        }
        Stage::Review => {
            requirements.push(HumanRequirement::Review);
        }
        Stage::Test => {
            if tasks.completed < tasks.total {
                requirements.push(HumanRequirement::Test);
            }
        }
        _ => {}
    }
    
    // Add Fix requirement if there are uncommitted changes
    if !git_status.clean {
        requirements.push(HumanRequirement::Fix);
    }
    
    // Add Decision requirement if there are blocked tasks
    if tasks.blocked > 0 {
        requirements.push(HumanRequirement::Decision);
    }
    
    requirements
}

/// Check if there are error markers in build/test output files
pub fn has_error_markers(_project_path: &std::path::Path) -> bool {
    // This is a simplified check - in reality would scan actual files
    // For now, return false to avoid false positives
    // TODO: Implement actual file scanning for error patterns
    false
}