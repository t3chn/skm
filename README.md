# SKM - Spec-Kit Manager

> Intelligent portfolio management for development projects following the Spec-Kit methodology

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

SKM automatically discovers, analyzes, and prioritizes your development projects, helping you focus on what matters most.

## ✨ Key Features

- 🔍 **Smart Discovery** - Automatically finds all Spec-Kit projects in your workspace
- 📊 **Dual Structure Support** - Works with both `.specify/` and `specs/` directory layouts
- ✅ **Flexible Task Tracking** - Parses multiple formats: checkboxes, IDs (T001:), emojis (✅❌), keywords
- 🎯 **Intelligent Prioritization** - Multi-factor scoring based on urgency, risk, and impact
- 📈 **Progress Tracking** - Automatic stage detection across project lifecycle
- ⚡ **High Performance** - Optimized parsing (50-100x faster than naive approaches)
- 📝 **Rich Reporting** - Generate Markdown and JSON reports
- 🔄 **Smart Caching** - Fast status updates with intelligent cache invalidation

## 🚀 Quick Start

### Installation

```bash
# Clone and build
git clone https://github.com/t3chn/skm.git
cd skm
cargo build --release

# Optional: Install globally
cargo install --path .
```

### Basic Usage

```bash
# Scan your projects
skm scan --root ~/projects

# View portfolio status
skm status

# Filter high-priority projects
skm status --only needs-attention

# Enable detailed logging
SKM_DEBUG=1 skm scan
```

### Example Output

```
Found: crypto-trader [Implement] Priority: 61.6
Found: web-dashboard [Review] Priority: 45.3
Found: api-service [Test] Priority: 38.9

=== Scan Complete ===
Projects found: 15
Need attention: 12
Tasks: 510/852 completed (59.9%)
Average priority: 55.3
Scan time: 1670ms
```

## 📖 How It Works

### Project Discovery

SKM scans your workspace for Spec-Kit projects and supports two structures:

**Standard Layout:**
```
my-project/
├── .specify/
│   ├── constitution.md
│   ├── spec.md
│   ├── plan.md
│   └── tasks.md
```

**Feature-Based Layout:**
```
my-project/
├── specs/
│   ├── 001-user-authentication/
│   │   ├── spec.md
│   │   ├── plan.md
│   │   └── tasks.md
│   ├── 002-payment-processing/
│   │   └── ...
```

### Task Format Support

SKM understands multiple task formats:

```markdown
- [ ] Standard checkbox
- [x] Completed checkbox
- [ ] T001: Task with ID
T002: Standalone task ID
✅ Emoji completed
❌ Emoji incomplete
🔄 In progress
TODO: Keyword format
DONE: Completed keyword
```

Special markers:
- `[P]` or `||` - Parallel execution
- `[BLOCKED]` or 🚫 - Blocked task

### Priority Calculation

Priority score uses weighted formula:

```
Score = w₁×NeedsHuman + w₂×Risk + w₃×Staleness + w₄×Impact - w₅×Confidence
```

Default weights:
- Human attention needed: 40%
- Risk level: 25%
- Time since update: 15%
- Project impact: 15%
- Confidence: -10%

## 🎯 Commands

### Core Commands

#### `scan` - Discover and analyze projects

```bash
skm scan                           # Scan current directory
skm scan --root /path/to/projects  # Scan specific location
```

Generates:
- `.skm/STATUS.md` - Markdown report
- `.skm/status.json` - Cached data

#### `status` - View portfolio overview

```bash
skm status                         # Show all projects
skm status --json                  # JSON output
skm status --only needs-attention  # Filter high-priority
skm status --only incomplete       # Filter active tasks
skm status --only stage:implement  # Filter by stage
```

#### `report` - Generate formatted reports

```bash
skm report --format md             # Markdown (default)
skm report --format json           # JSON export
skm report --format table          # Terminal table
skm report --out custom.md         # Custom output path
```

## ⚙️ Configuration

SKM looks for configuration at `~/.config/skm/config.toml`:

```toml
# Priority calculation weights
[weights]
needs_human = 40.0    # Human attention required
risk = 25.0           # Risk assessment
staleness = 15.0      # Days since update
impact = 15.0         # Project importance
confidence = 10.0     # Solution certainty

# General settings
attention_threshold = 50.0  # Priority threshold for "needs attention"
scan_depth = 5              # Maximum directory depth
default_editor = "nvim"     # Editor for manual edits

# External services (future)
qdrant_url = "http://localhost:6333"
automation_level = "L1"
```

### Project-Specific Metadata

Store per-project settings in `.skm/meta.json`:

```json
{
  "projects": {
    "critical-service": {
      "impact": 3,              # 1-3 scale
      "approved_by_human": true,
      "automation_level": "L2"
    }
  }
}
```

## 🏗️ Architecture

```
skm/
├── src/
│   ├── scanner/       # Project discovery & parsing
│   ├── analyzer/      # Stage detection & prioritization
│   ├── reporter/      # Report generation
│   ├── meta/          # Configuration & state management
│   ├── rag/           # Vector search (future)
│   ├── autopilot/     # Automation engine (future)
│   └── session/       # Session management (future)
```

### Stage Lifecycle

```
Bootstrap → Specify → Plan → Tasks → Implement → Test → Review → Done
```

Each stage has specific requirements and next actions.

## 🛠️ Development

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- scan

# Format code
cargo fmt

# Lint
cargo clippy

# Build release
cargo build --release
```

## 📊 Performance

- **Scan Speed**: ~1-2s for 15 projects
- **Task Parsing**: 50-100x faster with optimized regex
- **Memory**: Minimal overhead, efficient caching
- **Disk Usage**: Status cache < 100KB

## 🗺️ Roadmap

- [x] Core scanning and prioritization
- [x] Multiple task format support
- [x] Feature-based directory support
- [x] Status caching
- [ ] RAG-based semantic search
- [ ] Automation engine with safety levels
- [ ] tmux session management
- [ ] GitHub integration
- [ ] Interactive TUI mode
- [ ] Watch mode for continuous monitoring

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

Built for the [Spec-Kit](https://github.com/spec-kit/spec-kit) methodology by [@klueless-io](https://github.com/klueless-io)

---

**Made with ❤️ using Rust and [Claude Code](https://claude.com/claude-code)**
