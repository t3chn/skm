use crate::{Stage, NextAction, AutomationLevel, ArtifactStatus, ProjectType};

/// Detect the current stage of a project based on artifacts
pub fn detect_stage(artifacts: &ArtifactStatus, project_type: &ProjectType) -> Stage {
    if artifacts.constitution.is_none() {
        return Stage::Bootstrap;
    }
    
    if artifacts.spec.is_none() {
        return Stage::Specify;
    }
    
    if artifacts.plan.is_none() {
        return Stage::Plan;
    }
    
    if artifacts.tasks.is_none() {
        return Stage::Tasks;
    }
    
    // Check for implementation artifacts
    if !has_implementation_artifacts(artifacts, project_type) {
        return Stage::Implement;
    }
    
    // If we have all artifacts and implementation, we're in test/review stage
    // This would need more sophisticated detection in a real implementation
    Stage::Test
}

/// Determine the next action based on the current stage
pub fn get_next_action(stage: &Stage) -> NextAction {
    match stage {
        Stage::Bootstrap => NextAction {
            command: "/speckit.constitution".to_string(),
            description: "Create project constitution to establish core values and principles".to_string(),
            automated: false,
            risk_level: AutomationLevel::L2,
        },
        Stage::Specify => NextAction {
            command: "/speckit.specify".to_string(),
            description: "Create specification with user stories and requirements".to_string(),
            automated: false,
            risk_level: AutomationLevel::L2,
        },
        Stage::Plan => NextAction {
            command: "/speckit.plan".to_string(),
            description: "Create implementation plan with technical design".to_string(),
            automated: false,
            risk_level: AutomationLevel::L2,
        },
        Stage::Tasks => NextAction {
            command: "/speckit.tasks".to_string(),
            description: "Generate task breakdown for implementation".to_string(),
            automated: true,
            risk_level: AutomationLevel::L1,
        },
        Stage::Implement => NextAction {
            command: "/speckit.implement".to_string(),
            description: "Begin implementation of tasks".to_string(),
            automated: false,
            risk_level: AutomationLevel::L3,
        },
        Stage::Test => NextAction {
            command: "Run tests and verify implementation".to_string(),
            description: "Execute test suite and validate functionality".to_string(),
            automated: true,
            risk_level: AutomationLevel::L1,
        },
        Stage::Review => NextAction {
            command: "Review code and documentation".to_string(),
            description: "Perform code review and quality checks".to_string(),
            automated: false,
            risk_level: AutomationLevel::L1,
        },
        Stage::Done => NextAction {
            command: "Project complete".to_string(),
            description: "All stages completed successfully".to_string(),
            automated: false,
            risk_level: AutomationLevel::L0,
        },
    }
}

fn has_implementation_artifacts(_artifacts: &ArtifactStatus, project_type: &ProjectType) -> bool {
    // For now, we'll use a simple heuristic
    // In reality, this would check for actual source files
    match project_type {
        ProjectType::Rust => {
            // Would check for src/*.rs files
            false
        },
        ProjectType::Node => {
            // Would check for src/*.js or src/*.ts files
            false
        },
        ProjectType::Python => {
            // Would check for src/*.py files
            false
        },
        ProjectType::Go => {
            // Would check for *.go files
            false
        },
        _ => false,
    }
}

/// Check if a project needs immediate human attention
pub fn needs_human_attention(stage: &Stage) -> bool {
    matches!(stage, 
        Stage::Bootstrap | 
        Stage::Specify | 
        Stage::Plan | 
        Stage::Review
    )
}

/// Get a human-readable description of the stage
pub fn stage_description(stage: &Stage) -> &'static str {
    match stage {
        Stage::Bootstrap => "Needs constitution - establish project identity",
        Stage::Specify => "Needs specification - define requirements",
        Stage::Plan => "Needs plan - design technical approach",
        Stage::Tasks => "Needs tasks - break down work items",
        Stage::Implement => "In implementation - coding in progress",
        Stage::Test => "In testing - validating functionality",
        Stage::Review => "In review - awaiting approval",
        Stage::Done => "Complete - all stages finished",
    }
}