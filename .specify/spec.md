# SKM Specification

## User Stories

### US-A: Portfolio Manager - Daily Overview
**As a** portfolio manager managing 50+ projects  
**I want** a daily overview of all my Spec-Kit projects  
**So that** I can prioritize where to focus my attention  

**Acceptance Criteria:**
- [x] Scan finds all projects with `.specify` directories
- [x] Priority score highlights projects needing attention
- [x] Status report shows stage distribution
- [x] Human intervention requirements clearly marked
- [x] Scan completes in under 5 seconds for 100 projects

### US-B: Developer - Contextual Q&A
**As a** developer jumping between projects  
**I want** to query project status and history  
**So that** I can quickly understand context  

**Acceptance Criteria:**
- [x] RAG system indexes all project artifacts
- [x] Natural language queries supported
- [x] Relevant snippets returned with sources
- [x] Cross-project insights available

### US-C: Team Lead - Automation Control
**As a** team lead with CI/CD responsibilities  
**I want** controlled automation with approval gates  
**So that** routine tasks are automated safely  

**Acceptance Criteria:**
- [x] L0-L3 automation levels implemented
- [x] Approval workflow for risky operations
- [x] Dry-run mode for all commands
- [x] Audit log of all automated actions

## Technical Requirements

### Performance
- Scan 100 projects in < 5 seconds
- RAG query response in < 2 seconds
- Status cache invalidation < 100ms

### Reliability
- Graceful handling of malformed projects
- Atomic operations for state changes
- Rollback capability for automation

### Integration
- Git-aware operations
- tmux session management
- Claude/GPT handoff protocol
- Qdrant vector database