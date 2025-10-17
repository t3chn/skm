# SKM Implementation Plan

## Architecture Overview

```
skm/
├── scanner/       # Project discovery and parsing
├── analyzer/      # Stage detection and prioritization  
├── reporter/      # Report generation (MD, JSON, table)
├── meta/          # Configuration and state management
├── rag/           # Vector search with Qdrant
├── autopilot/     # Automation engine (L0-L3)
└── session/       # tmux and handoff management
```

## Implementation Phases

### Phase 1: Foundation (COMPLETE)
- [x] Rust project setup
- [x] Core data structures
- [x] Error handling
- [x] CLI framework

### Phase 2: Scanning & Analysis (COMPLETE)
- [x] Project discovery via `.specify`
- [x] Artifact parsing (constitution, spec, plan, tasks)
- [x] Git status integration
- [x] Stage detection algorithm
- [x] Priority calculation with weights

### Phase 3: Reporting & Status (COMPLETE)
- [x] Markdown report generation
- [x] JSON status caching
- [x] Portfolio aggregation
- [x] CLI display formatting

### Phase 4: RAG Integration (COMPLETE)
- [x] Qdrant client setup
- [x] Document embedding (simplified)
- [x] Semantic search
- [x] Q&A interface

### Phase 5: Automation (COMPLETE)
- [x] L0-L3 level definitions
- [x] Command classification
- [x] Approval workflow
- [x] Execution engine

### Phase 6: Session Management (COMPLETE)
- [x] tmux integration
- [x] Project sessions
- [x] Handoff protocol
- [x] Context serialization

## Key Algorithms

### Priority Scoring
```
Priority = w1*NeedsHuman + w2*Risk + w3*Staleness + w4*Impact - w5*Confidence
```

### Stage Progression
```
Bootstrap → Specify → Plan → Tasks → Implement → Test → Review → Done
```

## Integration Points

- **Git**: Status, branch, commits
- **Qdrant**: Vector storage at localhost:6333
- **tmux**: Session management
- **File System**: Project scanning