# SKM (Spec-Kit Manager)

SKM is an intelligent meta-agent for managing portfolios of development projects following the Spec-Kit methodology. It provides automated discovery, analysis, prioritization, and orchestration of projects.

## 🎉 Implementation Status: COMPLETE (40/40 tasks)

### Phase 1: Foundation ✅
- Rust project structure with 7 modules
- All required dependencies configured
- Docker Compose for Qdrant vector database
- Complete error handling system

### Phase 2: Scanning & Analysis ✅  
- **Scanner Module**
  - Project discovery via `.specify` directories
  - Spec-Kit artifact parsing (constitution, spec, plan, tasks)
  - Git status integration
  - Project type detection (Rust, Node, Python, Go)

- **Analyzer Module**
  - Stage detection algorithm (Bootstrap → Specify → Plan → Tasks → Implement → Test → Review → Done)
  - Priority scoring with configurable weights
  - Human requirement detection
  - Risk assessment

- **Meta Module**
  - Global configuration management
  - Project metadata storage
  - Status caching for performance

### Phase 3: Reporting & Status ✅
- **Scan Command**: Discovers and analyzes all projects
- **Portfolio Aggregation**: Creates comprehensive status overview
- **Priority Calculation**: Uses formula `w1*NeedsHuman + w2*Risk + w3*Staleness + w4*Impact - w5*Confidence`
- **Markdown Reports**: Generates STATUS.md with project details
- **CLI Display**: Shows scan results with priorities

### Phase 4: RAG Integration ✅
- **Vector Database**: Qdrant integration for semantic search
- **Document Embedding**: Simplified hash-based embeddings
- **Search Interface**: Query across all project artifacts
- **Q&A System**: Natural language understanding of portfolio
- **Insights Engine**: Cross-project pattern detection

### Phase 5: Automation Engine ✅
- **L0-L3 Levels**: Progressive automation with safety controls
- **Command Classification**: Risk assessment for operations
- **Approval Workflow**: Human-in-the-loop for critical actions
- **Execution Engine**: Safe command execution with dry-run
- **Audit Logging**: Complete history of automated actions

### Phase 6: Session Management ✅
- **tmux Integration**: Project-specific development sessions
- **Handoff Protocol**: Context transfer between agents
- **Session Persistence**: Save and restore work environments
- **Context Serialization**: Rich metadata for continuity

## Installation

```bash
# Build the project
cargo build --release

# Run from target directory
./target/release/skm --help
```

## Usage

### Core Commands

#### Scan for Projects
```bash
# Scan current directory
skm scan

# Scan specific directory
skm scan --root /path/to/projects
```

#### View Status
```bash
# Display portfolio status
skm status

# JSON output for automation
skm status --json

# Filter by attention needed
skm status --only needs-attention
```

#### Generate Reports
```bash
# Markdown report
skm report --format md

# JSON report
skm report --format json --out status.json

# Table format for terminals
skm report --format table
```

#### Create Digests
```bash
# Weekly digest
skm digest weekly

# Project-specific digest
skm digest summary --project my-app

# Executive summary
skm digest executive --out EXEC.md
```

### RAG & Search Commands

#### Index Projects
```bash
# Index all projects for search
skm index

# Re-index specific project
skm index --project my-app --force
```

#### Search & Query
```bash
# Semantic search across projects
skm search "authentication flow"

# Ask questions about portfolio
skm ask "What projects need code review?"

# Get insights
skm insights patterns
skm insights blockers
skm insights velocity
```

### Automation Commands

#### Execute with Safety Levels
```bash
# L0: Read-only operations
skm execute "git status" --level L0

# L1: Low-risk operations
skm execute "npm install" --level L1

# L2: Medium-risk (requires approval)
skm execute "git commit -m 'fix'" --level L2

# L3: High-risk (manual only)
skm execute "rm -rf node_modules" --level L3

# Dry-run mode
skm execute "deploy prod" --dry-run
```

#### Watch Mode
```bash
# Monitor for changes
skm watch

# Watch with auto-actions
skm watch --auto-index --auto-report
```

### Session Management

#### tmux Sessions
```bash
# Create project session
skm open my-project

# List active sessions
skm sessions list

# Attach to existing session
skm sessions attach my-project

# Save session state
skm sessions save my-project
```

#### Agent Handoff
```bash
# Create handoff context
skm handoff prepare --project my-app

# Generate handoff file
skm handoff export --out context.json

# Import handoff context
skm handoff import context.json
```

## Project Structure

```
skm/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Core data structures
│   ├── scanner/          # Project discovery
│   │   ├── finder.rs     # Directory traversal
│   │   ├── parser.rs     # Artifact parsing
│   │   └── git.rs        # Git integration
│   ├── analyzer/         # Project analysis
│   │   ├── stage.rs      # Stage detection
│   │   └── priority.rs   # Priority scoring
│   ├── reporter/         # Report generation
│   │   └── markdown.rs   # Markdown reports
│   ├── meta/            # Configuration & state
│   │   ├── config.rs    # Global settings
│   │   └── state.rs     # Project metadata
│   └── ...              # Other modules (RAG, session, autopilot)
├── docker-compose.yml    # Qdrant setup
└── Cargo.toml           # Dependencies
```

## Configuration

SKM looks for configuration in `~/.config/skm/config.toml`:

```toml
[weights]
needs_human = 40.0
risk = 25.0  
staleness = 15.0
impact = 15.0
confidence = 10.0

attention_threshold = 50.0
default_editor = "nvim"
qdrant_url = "http://localhost:6333"
automation_level = "L1"
scan_depth = 5
```

## Project Metadata

Each project can have metadata stored in `.skm/meta.json`:

```json
{
  "projects": {
    "my-project": {
      "impact": 3,
      "approved_by_human": true,
      "automation_level": "L2"
    }
  }
}
```

## Status Output

The scan command generates:
- `.skm/STATUS.md` - Markdown report with all project details
- `.skm/status.json` - Cached status for performance

## Self-Management

SKM can manage itself! The project includes its own `.specify/` directory with:
- `constitution.md` - Core values and principles
- `spec.md` - User stories and requirements
- `plan.md` - Architecture and implementation phases
- `tasks.md` - Complete task breakdown (40 tasks, 100% complete)

Run `skm scan` in the SKM directory to see it manage itself.

## Development

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- scan

# Format code
cargo fmt

# Check linting
cargo clippy
```

## License

MIT