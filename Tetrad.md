# Tetrad: Quadruple Consensus MCP Server in Rust

> **Version 2.0** - Revised with learnings from Claude-Flow

## Executive Summary

**Tetrad** is a high-performance MCP server written in Rust that orchestrates three CLI code tools (Codex, Gemini CLI, Qwen Code) to evaluate and validate all work produced by Claude Code. The system implements a quadruple consensus protocol where no code or plan is accepted without unanimous approval from four intelligences: the three external evaluators + Claude Code itself.

### What's New in v2.0 (Inspired by Claude-Flow)

| Feature                  | Description                                                                      |
| ------------------------ | -------------------------------------------------------------------------------- |
| **ReasoningBank**        | Continuous learning system with RETRIEVE→JUDGE→DISTILL→CONSOLIDATE cycle        |
| **Interactive CLI**      | Commands `tetrad init`, `tetrad status`, `tetrad config`                         |
| **crates.io Distribution** | `cargo install tetrad` for global installation                                 |
| **Plugin System**        | Extensibility for new evaluators                                                 |
| **CLAUDE.md**            | Documentation for Claude Code to use automatically                               |
| **Hooks**                | Callbacks for pre/post evaluation                                                |
| **Persistence**          | SQLite for cross-session history                                                 |

### Why Rust?

| Aspect                | Benefit                                                    |
| --------------------- | ---------------------------------------------------------- |
| **Performance**       | Native parallel execution with zero runtime overhead       |
| **Reliability**       | Type system that prevents bugs at compile time             |
| **Concurrency**       | Tokio async runtime for simultaneous CLI calls             |
| **Single Binary**     | Simple deployment without runtime dependencies             |
| **Low Latency**       | Ideal for MCP that needs to respond quickly                |
| **crates.io**         | Easy distribution like npm for Node.js                     |

---

## 1. Installation and Usage (Like Claude-Flow)

### 1.1 Quick Installation

```bash
# Via cargo (recommended)
cargo install tetrad

# Via Homebrew (macOS/Linux)
brew install tetrad

# Via direct binary (GitHub releases)
curl -fsSL https://github.com/SamoraDC/tetrad/releases/latest/download/install.sh | sh
```

### 1.2 Initialization

```bash
# Initialize configuration in current project
tetrad init

# Check CLI status
tetrad status

# Configure interactively
tetrad config
```

### 1.3 Integration with Claude Code

```bash
# Add as MCP server (available in all projects)
claude mcp add --scope user tetrad -- tetrad serve

# Or for current project only
claude mcp add tetrad -- tetrad serve

# Verify it's configured
claude mcp list
```

### 1.4 Available CLI Commands

```
tetrad - Quadruple Consensus CLI for Claude Code

USAGE:
    tetrad <COMMAND>

COMMANDS:
    init        Initialize configuration in current directory
    serve       Start MCP server (used by Claude Code)
    status      Show CLI status (codex, gemini, qwen)
    config      Configure options interactively
    evaluate    Evaluate code manually (without MCP)
    history     Show evaluation history
    export      Export ReasoningBank to file
    import      Import patterns from another ReasoningBank
    doctor      Diagnose configuration issues
    version     Show version

OPTIONS:
    -c, --config <FILE>    Configuration file (default: tetrad.toml)
    -v, --verbose          Verbose mode
    -q, --quiet            Quiet mode
    -h, --help             Show help
```

---

## 2. System Architecture

### 2.1 Overview (Updated)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLAUDE CODE                                     │
│                      (Code Generator + Final Decision Maker)                 │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │ MCP Protocol (stdio)
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       MCP SERVER: TETRAD (Rust)                              │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                           ORCHESTRATOR                                  │ │
│  │  • Receives MCP requests from Claude Code                              │ │
│  │  • Manages gate pipeline (Plan → Impl → Tests)                         │ │
│  │  • Coordinates refinement loop until consensus                         │ │
│  │  • Queries ReasoningBank for known patterns                            │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                │                                             │
│          ┌─────────────────────┼─────────────────────┐                      │
│          ▼                     ▼                     ▼                      │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐                │
│  │    CODEX     │     │    GEMINI    │     │     QWEN     │                │
│  │   Executor   │     │   Executor   │     │   Executor   │                │
│  │              │     │              │     │              │                │
│  │ CLI: codex   │     │ CLI: gemini  │     │ CLI: qwen    │                │
│  │ exec --json  │     │ -o json      │     │              │                │
│  └──────────────┘     └──────────────┘     └──────────────┘                │
│          │                     │                     │                      │
│          └─────────────────────┼─────────────────────┘                      │
│                                ▼                                             │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                       CONSENSUS ENGINE                                  │ │
│  │  • Collects votes (PASS/WARN/FAIL) from each CLI                       │ │
│  │  • Applies rules: Golden Rule, Weak/Strong Consensus                   │ │
│  │  • Calculates aggregate score and confidence                           │ │
│  │  • Generates consolidated, actionable feedback                         │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                │                                             │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                     REASONING BANK (SQLite)                             │ │
│  │                                                                         │ │
│  │  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────────┐        │ │
│  │  │ RETRIEVE │──▶│  JUDGE   │──▶│ DISTILL  │──▶│ CONSOLIDATE  │        │ │
│  │  │          │   │          │   │          │   │              │        │ │
│  │  │ Search   │   │ Evaluate │   │ Extract  │   │ Prevent      │        │ │
│  │  │ similar  │   │ success/ │   │ learnings│   │ forgetting   │        │ │
│  │  │ patterns │   │ failure  │   │          │   │              │        │ │
│  │  └──────────┘   └──────────┘   └──────────┘   └──────────────┘        │ │
│  │                                                                         │ │
│  │  • Cross-session persistence (SQLite)                                  │ │
│  │  • Exportable/Importable for sharing patterns                          │ │
│  │  • Prevents repetition of known errors                                 │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                          HOOK SYSTEM                                    │ │
│  │  • pre_evaluate: Before sending to CLIs                                │ │
│  │  • post_evaluate: After receiving votes                                │ │
│  │  • on_consensus: When consensus is reached                             │ │
│  │  • on_block: When evaluation is blocked                                │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         PLUGIN SYSTEM                                   │ │
│  │  • New executors (e.g.: local Claude, Llama, etc.)                     │ │
│  │  • New exporters (JSON, CSV, Markdown)                                 │ │
│  │  • Integrations (GitHub, GitLab, Jira)                                 │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Data Flow with ReasoningBank

```
┌──────────┐    ┌──────────┐    ┌───────────┐    ┌──────────┐    ┌──────────┐
│  Claude  │    │ Reasoning│    │    3      │    │ Consensus│    │  Claude  │
│   Code   │───▶│   Bank   │───▶│   CLIs    │───▶│  Engine  │───▶│   Code   │
│  (input) │    │(RETRIEVE)│    │(parallel) │    │ (aggr)   │    │ (output) │
└──────────┘    └──────────┘    └───────────┘    └──────────┘    └──────────┘
     │                                                 │
     │                                                 ▼
     │                                        ┌──────────────┐
     │                                        │ ReasoningBank│
     │                                        │   (JUDGE +   │
     │                                        │   DISTILL +  │
     │                                        │ CONSOLIDATE) │
     │                                        └──────────────┘
     │                                                 │
     │              REFINEMENT LOOP                    │
     └─────────────────────────────────────────────────┘
```

---

## 3. ReasoningBank: Continuous Learning System

Inspired by Claude-Flow, the ReasoningBank implements a learning cycle that improves evaluations over time.

### 3.1 The RETRIEVE → JUDGE → DISTILL → CONSOLIDATE Cycle

```rust
// src/reasoning/bank.rs

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// ReasoningBank - Continuous learning system inspired by Claude-Flow
pub struct ReasoningBank {
    conn: Connection,
    config: ReasoningConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: i64,
    pub pattern_type: PatternType,
    pub code_signature: String,      // Hash or fingerprint of the code
    pub language: String,
    pub issue_category: String,      // "security", "logic", "performance", etc.
    pub description: String,
    pub solution: Option<String>,
    pub success_count: i32,          // How many times the pattern led to success
    pub failure_count: i32,          // How many times the pattern led to failure
    pub confidence: f64,             // Calculated: success / (success + failure)
    pub last_seen: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    AntiPattern,    // Code that always fails
    GoodPattern,    // Code that always passes
    Ambiguous,      // Mixed results
}

impl ReasoningBank {
    /// Creates or opens the pattern bank
    pub fn new(db_path: &str) -> anyhow::Result<Self> {
        let conn = Connection::open(db_path)?;

        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pattern_type TEXT NOT NULL,
                code_signature TEXT NOT NULL,
                language TEXT NOT NULL,
                issue_category TEXT NOT NULL,
                description TEXT NOT NULL,
                solution TEXT,
                success_count INTEGER DEFAULT 0,
                failure_count INTEGER DEFAULT 0,
                confidence REAL DEFAULT 0.5,
                last_seen TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(code_signature, issue_category)
            );

            CREATE TABLE IF NOT EXISTS trajectories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pattern_id INTEGER REFERENCES patterns(id),
                request_id TEXT NOT NULL,
                code_hash TEXT NOT NULL,
                initial_score INTEGER,
                final_score INTEGER,
                loops_to_consensus INTEGER,
                was_successful BOOLEAN,
                timestamp TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_patterns_signature ON patterns(code_signature);
            CREATE INDEX IF NOT EXISTS idx_patterns_category ON patterns(issue_category);
            CREATE INDEX IF NOT EXISTS idx_trajectories_pattern ON trajectories(pattern_id);
        "#)?;

        Ok(Self {
            conn,
            config: ReasoningConfig::default(),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 1: RETRIEVE - Search for similar patterns
    // ═══════════════════════════════════════════════════════════════════════

    /// Searches for known patterns that may affect the evaluation
    pub fn retrieve(&self, code: &str, language: &str) -> Vec<PatternMatch> {
        let signature = self.compute_signature(code);
        let keywords = self.extract_keywords(code);

        let mut matches = Vec::new();

        // Search by exact signature
        if let Ok(exact) = self.find_by_signature(&signature) {
            matches.extend(exact.into_iter().map(|p| PatternMatch {
                pattern: p,
                match_type: MatchType::Exact,
                relevance: 1.0,
            }));
        }

        // Search by keywords (known problem patterns)
        for keyword in &keywords {
            if let Ok(keyword_matches) = self.find_by_keyword(keyword, language) {
                matches.extend(keyword_matches.into_iter().map(|p| PatternMatch {
                    relevance: 0.7, // Less reliable than exact match
                    pattern: p,
                    match_type: MatchType::Keyword,
                }));
            }
        }

        // Sort by relevance and confidence
        matches.sort_by(|a, b| {
            let score_a = a.relevance * a.pattern.confidence;
            let score_b = b.relevance * b.pattern.confidence;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return top N matches
        matches.truncate(self.config.max_patterns_per_query);
        matches
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 2: JUDGE - Evaluate success/failure of the evaluation
    // ═══════════════════════════════════════════════════════════════════════

    /// Judges the result of an evaluation and updates patterns
    pub fn judge(
        &mut self,
        request_id: &str,
        code: &str,
        language: &str,
        result: &EvaluationResult,
        loops_to_consensus: u32,
    ) -> anyhow::Result<JudgmentResult> {
        let signature = self.compute_signature(code);
        let was_successful = result.consensus_achieved && loops_to_consensus <= 2;

        // Record trajectory
        let trajectory = Trajectory {
            request_id: request_id.to_string(),
            code_hash: signature.clone(),
            initial_score: result.votes.values().map(|v| v.score).min().unwrap_or(0),
            final_score: result.score,
            loops_to_consensus,
            was_successful,
            timestamp: Utc::now(),
        };

        // For each issue found, update or create pattern
        for finding in &result.findings {
            self.update_or_create_pattern(
                &signature,
                language,
                &finding.issue,
                finding.suggestion.as_deref(),
                &finding.severity,
                was_successful,
            )?;
        }

        // If there were no issues and it was a success, register as GoodPattern
        if result.findings.is_empty() && was_successful {
            self.register_good_pattern(&signature, language)?;
        }

        self.save_trajectory(&trajectory)?;

        Ok(JudgmentResult {
            was_successful,
            patterns_updated: result.findings.len(),
            new_patterns_created: 0, // Will be updated by the method
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 3: DISTILL - Extract learnings from patterns
    // ═══════════════════════════════════════════════════════════════════════

    /// Distills knowledge from patterns to generate insights
    pub fn distill(&self) -> DistilledKnowledge {
        // Top anti-patterns (most failures)
        let top_antipatterns = self.get_top_patterns(PatternType::AntiPattern, 10);

        // Top good patterns (most successes)
        let top_good_patterns = self.get_top_patterns(PatternType::GoodPattern, 10);

        // Most problematic categories
        let problematic_categories = self.get_problematic_categories();

        // Languages with most issues
        let language_stats = self.get_language_stats();

        // Average time to consensus
        let avg_loops = self.get_average_loops_to_consensus();

        DistilledKnowledge {
            top_antipatterns,
            top_good_patterns,
            problematic_categories,
            language_stats,
            avg_loops_to_consensus: avg_loops,
            total_patterns: self.count_patterns(),
            total_trajectories: self.count_trajectories(),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 4: CONSOLIDATE - Prevent forgetting important patterns
    // ═══════════════════════════════════════════════════════════════════════

    /// Consolidates knowledge, preventing forgetting of important patterns
    pub fn consolidate(&mut self) -> anyhow::Result<ConsolidationResult> {
        let mut merged = 0;
        let mut pruned = 0;
        let mut reinforced = 0;

        // Merge similar patterns
        merged += self.merge_similar_patterns()?;

        // Remove patterns with low confidence and little use
        pruned += self.prune_low_quality_patterns()?;

        // Reinforce patterns that consistently prevent errors
        reinforced += self.reinforce_high_value_patterns()?;

        // Update confidence for all patterns
        self.recalculate_all_confidences()?;

        Ok(ConsolidationResult {
            patterns_merged: merged,
            patterns_pruned: pruned,
            patterns_reinforced: reinforced,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Auxiliary methods
    // ═══════════════════════════════════════════════════════════════════════

    fn compute_signature(&self, code: &str) -> String {
        use sha2::{Sha256, Digest};

        // Normalize code (remove extra whitespace, comments)
        let normalized = self.normalize_code(code);

        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn normalize_code(&self, code: &str) -> String {
        code.lines()
            .map(|l| l.trim())
            .filter(|l| !l.starts_with("//") && !l.starts_with("#") && !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn extract_keywords(&self, code: &str) -> Vec<String> {
        // Extract keywords that indicate known patterns
        let mut keywords = Vec::new();
        let code_lower = code.to_lowercase();

        // Security keywords
        if code_lower.contains("sql") || code_lower.contains("query") {
            keywords.push("sql".to_string());
        }
        if code_lower.contains("password") || code_lower.contains("secret") {
            keywords.push("credentials".to_string());
        }
        if code_lower.contains("eval") || code_lower.contains("exec") {
            keywords.push("code_execution".to_string());
        }

        // Logic keywords
        if code_lower.contains("for") || code_lower.contains("while") {
            keywords.push("loop".to_string());
        }
        if code_lower.contains("unwrap") || code_lower.contains(".get(") {
            keywords.push("null_access".to_string());
        }

        keywords
    }

    /// Exports ReasoningBank to share with others
    pub fn export(&self, path: &str) -> anyhow::Result<()> {
        let knowledge = self.distill();
        let patterns = self.get_all_patterns()?;

        let export = ReasoningBankExport {
            version: "2.0".to_string(),
            exported_at: Utc::now(),
            knowledge,
            patterns,
        };

        let json = serde_json::to_string_pretty(&export)?;
        std::fs::write(path, json)?;

        Ok(())
    }

    /// Imports patterns from another ReasoningBank
    pub fn import(&mut self, path: &str) -> anyhow::Result<ImportResult> {
        let json = std::fs::read_to_string(path)?;
        let export: ReasoningBankExport = serde_json::from_str(&json)?;

        let mut imported = 0;
        let mut skipped = 0;

        for pattern in export.patterns {
            if self.pattern_exists(&pattern.code_signature, &pattern.issue_category)? {
                // Merge with existing pattern
                self.merge_imported_pattern(&pattern)?;
                skipped += 1;
            } else {
                // Import new pattern
                self.insert_pattern(&pattern)?;
                imported += 1;
            }
        }

        Ok(ImportResult { imported, skipped })
    }
}
```

### 3.2 Integration with Evaluations

```rust
// src/mcp/server.rs (updated)

impl TetradServer {
    pub async fn evaluate(&self, request: EvaluationRequest) -> Result<EvaluationResult, ServerError> {
        let start = std::time::Instant::now();

        // ═══════════════════════════════════════════════════════════════════
        // RETRIEVE PHASE: Search for known patterns
        // ═══════════════════════════════════════════════════════════════════
        let known_patterns = {
            let bank = self.reasoning_bank.read().await;
            bank.retrieve(&request.code, &request.language)
        };

        // If there are known anti-patterns, add to context
        let enriched_context = self.enrich_context_with_patterns(&request, &known_patterns);

        // Execute pre_evaluate hooks
        self.hooks.run_pre_evaluate(&request).await?;

        // ═══════════════════════════════════════════════════════════════════
        // Parallel evaluation in 3 models
        // ═══════════════════════════════════════════════════════════════════
        let (codex_result, gemini_result, qwen_result) = tokio::join!(
            self.execute_with_fallback(&*self.codex, &request, &enriched_context),
            self.execute_with_fallback(&*self.gemini, &request, &enriched_context),
            self.execute_with_fallback(&*self.qwen, &request, &enriched_context)
        );

        // Collect votes
        let votes = self.collect_votes(codex_result, gemini_result, qwen_result)?;

        // Calculate consensus
        let result = self.consensus.aggregate(&votes, &request);

        // Execute post_evaluate hooks
        self.hooks.run_post_evaluate(&request, &result).await?;

        // ═══════════════════════════════════════════════════════════════════
        // JUDGE + DISTILL + CONSOLIDATE PHASES
        // ═══════════════════════════════════════════════════════════════════
        {
            let mut bank = self.reasoning_bank.write().await;

            // JUDGE: Record result
            bank.judge(
                &request.request_id,
                &request.code,
                &request.language,
                &result,
                self.current_loop_count,
            )?;

            // CONSOLIDATE: Periodically (every N evaluations)
            if self.evaluation_count % self.config.consolidation_interval == 0 {
                bank.consolidate()?;
            }
        }

        // Consensus/block hooks
        if result.consensus_achieved {
            self.hooks.run_on_consensus(&result).await?;
        } else if result.decision == Decision::Block {
            self.hooks.run_on_block(&result).await?;
        }

        // Metrics
        let duration = start.elapsed();
        tracing::info!(
            request_id = %request.request_id,
            decision = ?result.decision,
            score = result.score,
            patterns_matched = known_patterns.len(),
            duration_ms = duration.as_millis(),
            "evaluation_completed"
        );

        Ok(result)
    }

    fn enrich_context_with_patterns(
        &self,
        request: &EvaluationRequest,
        patterns: &[PatternMatch],
    ) -> String {
        let mut context = request.context.clone().unwrap_or_default();

        if !patterns.is_empty() {
            context.push_str("\n\n## Known Patterns from ReasoningBank\n");
            context.push_str("The code has characteristics similar to known patterns:\n\n");

            for (i, pm) in patterns.iter().take(5).enumerate() {
                context.push_str(&format!(
                    "{}. **{}** (confidence: {:.0}%)\n   - {}\n",
                    i + 1,
                    pm.pattern.issue_category,
                    pm.pattern.confidence * 100.0,
                    pm.pattern.description
                ));

                if let Some(solution) = &pm.pattern.solution {
                    context.push_str(&format!("   - Suggested solution: {}\n", solution));
                }
            }

            context.push_str("\nPlease check these aspects especially.\n");
        }

        context
    }
}
```

---

## 4. Hook System

Inspired by Claude-Flow's 17 hooks, Tetrad offers a callback system for customization.

```rust
// src/hooks/mod.rs

use async_trait::async_trait;

#[async_trait]
pub trait Hook: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self, context: &HookContext) -> anyhow::Result<HookResult>;
}

pub struct HookSystem {
    pre_evaluate: Vec<Box<dyn Hook>>,
    post_evaluate: Vec<Box<dyn Hook>>,
    on_consensus: Vec<Box<dyn Hook>>,
    on_block: Vec<Box<dyn Hook>>,
}

impl HookSystem {
    pub fn new() -> Self {
        Self {
            pre_evaluate: Vec::new(),
            post_evaluate: Vec::new(),
            on_consensus: Vec::new(),
            on_block: Vec::new(),
        }
    }

    pub fn register(&mut self, event: HookEvent, hook: Box<dyn Hook>) {
        match event {
            HookEvent::PreEvaluate => self.pre_evaluate.push(hook),
            HookEvent::PostEvaluate => self.post_evaluate.push(hook),
            HookEvent::OnConsensus => self.on_consensus.push(hook),
            HookEvent::OnBlock => self.on_block.push(hook),
        }
    }

    pub async fn run_pre_evaluate(&self, request: &EvaluationRequest) -> anyhow::Result<()> {
        let context = HookContext::PreEvaluate { request };
        for hook in &self.pre_evaluate {
            hook.execute(&context).await?;
        }
        Ok(())
    }

    // ... other run_* methods
}

// Example of custom hook
pub struct LoggingHook;

#[async_trait]
impl Hook for LoggingHook {
    fn name(&self) -> &str {
        "logging"
    }

    async fn execute(&self, context: &HookContext) -> anyhow::Result<HookResult> {
        match context {
            HookContext::PreEvaluate { request } => {
                tracing::info!(request_id = %request.request_id, "Starting evaluation");
            }
            HookContext::PostEvaluate { result, .. } => {
                tracing::info!(
                    decision = ?result.decision,
                    score = result.score,
                    "Evaluation completed"
                );
            }
            _ => {}
        }
        Ok(HookResult::Continue)
    }
}
```

---

## 5. Plugin System

To allow extensibility like Claude-Flow.

```rust
// src/plugins/mod.rs

use async_trait::async_trait;

/// Trait for new evaluation executors
#[async_trait]
pub trait ExecutorPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn evaluate(&self, request: &EvaluationRequest) -> Result<ModelVote, PluginError>;
    fn specialization(&self) -> &str; // "syntax", "architecture", "logic", etc.
}

/// Trait for result exporters
pub trait ExporterPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn export(&self, results: &[EvaluationResult], path: &str) -> anyhow::Result<()>;
}

/// Plugin manager
pub struct PluginManager {
    executors: Vec<Box<dyn ExecutorPlugin>>,
    exporters: Vec<Box<dyn ExporterPlugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            executors: Vec::new(),
            exporters: Vec::new(),
        }
    }

    pub fn load_from_directory(&mut self, path: &str) -> anyhow::Result<()> {
        // Load plugins from .so/.dylib/.dll files
        // Or from WASM files for sandboxed plugins
        todo!("Plugin loading")
    }

    pub fn register_executor(&mut self, executor: Box<dyn ExecutorPlugin>) {
        tracing::info!(
            name = executor.name(),
            version = executor.version(),
            "Registered executor plugin"
        );
        self.executors.push(executor);
    }
}
```

---

## 6. CLAUDE.md - Documentation for Claude Code

Create this file in your project so Claude Code knows to use Tetrad automatically.

```markdown
# Tetrad Integration

This project uses **Tetrad** for quadruple consensus code validation.

## Configuration

Tetrad is configured as an MCP server. The available tools are:

### Mandatory Tools

You MUST use these tools before saving code:

1. **`tetrad_review_plan`** - Use BEFORE starting to implement
2. **`tetrad_review_code`** - Use BEFORE saving any file
3. **`tetrad_review_tests`** - Use BEFORE finalizing tests
4. **`tetrad_confirm`** - Use to confirm you agree with feedback
5. **`tetrad_final_check`** - Use after all corrections

### Workflow

```

1. Generate plan → tetrad_review_plan
2. If REVISE: adjust plan → tetrad_confirm → retry
3. Implement → tetrad_review_code
4. If REVISE: fix → tetrad_confirm → retry
5. Tests → tetrad_review_tests
6. tetrad_final_check → CERTIFIED

```

### Rules

- NEVER save code without going through tetrad_review_code
- If you receive BLOCK, stop and fix ALL critical issues
- If 2+ models point out the same problem, it's strong consensus - fix it
- Qwen focuses on logic bugs - pay special attention
- Gemini focuses on architecture - check impact on other modules
- Codex focuses on syntax - follow conventions

### ReasoningBank

Tetrad learns from past evaluations. If you receive a warning about
"known pattern", it means similar code has caused problems before.
Pay special attention to these warnings.
```

---

## 7. Project Structure (Updated)

```
tetrad/
├── Cargo.toml                    # Crate manifest
├── Cargo.lock
├── README.md                     # User documentation
├── CLAUDE.md                     # Documentation for Claude Code
├── LICENSE                       # MIT
├── src/
│   ├── main.rs                   # Entry point (CLI)
│   ├── lib.rs                    # Exportable library
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── commands.rs           # CLI commands (init, serve, status, etc.)
│   │   └── interactive.rs        # Interactive configuration
│   ├── mcp/
│   │   ├── mod.rs
│   │   ├── server.rs             # MCP server implementation
│   │   ├── protocol.rs           # MCP protocol types
│   │   ├── tools.rs              # Tool handlers
│   │   └── transport.rs          # stdio, HTTP transports
│   ├── executors/
│   │   ├── mod.rs
│   │   ├── base.rs               # Trait CliExecutor
│   │   ├── codex.rs              # Codex CLI wrapper
│   │   ├── gemini.rs             # Gemini CLI wrapper
│   │   └── qwen.rs               # Qwen CLI wrapper
│   ├── consensus/
│   │   ├── mod.rs
│   │   ├── engine.rs             # Consensus engine
│   │   ├── rules.rs              # Decision rules
│   │   └── aggregator.rs         # Vote aggregation
│   ├── reasoning/
│   │   ├── mod.rs
│   │   ├── bank.rs               # ReasoningBank (SQLite)
│   │   ├── patterns.rs           # Pattern matching
│   │   └── export.rs             # Export/Import
│   ├── hooks/
│   │   ├── mod.rs
│   │   └── builtin.rs            # Default hooks
│   ├── plugins/
│   │   ├── mod.rs
│   │   └── loader.rs             # Plugin loader
│   ├── prompts/
│   │   ├── mod.rs
│   │   ├── templates.rs          # Prompt templates
│   │   └── builders.rs           # Prompt builders
│   ├── cache/
│   │   ├── mod.rs
│   │   └── lru.rs                # LRU cache
│   └── types/
│       ├── mod.rs
│       ├── requests.rs           # Request types
│       ├── responses.rs          # Response types
│       ├── config.rs             # Configuration
│       └── errors.rs             # Error types
├── tests/
│   ├── integration/
│   │   ├── test_cli.rs
│   │   ├── test_mcp.rs
│   │   ├── test_reasoning.rs
│   │   └── test_consensus.rs
│   └── fixtures/
│       ├── good_code/
│       ├── bad_code/
│       └── patterns/
├── config/
│   └── default.toml              # Default configuration
├── plugins/                      # Example plugins
│   └── example_executor/
└── .github/
    └── workflows/
        ├── ci.yml                # CI/CD
        └── release.yml           # Release to crates.io
```

---

## 8. Cargo.toml (Updated for Distribution)

```toml
[package]
name = "tetrad"
version = "2.0.0"
edition = "2021"
authors = ["SamoraDC <samora@example.com>"]
description = "Quadruple Consensus MCP for Claude Code - Validates code using Codex, Gemini and Qwen"
license = "MIT"
repository = "https://github.com/SamoraDC/tetrad"
homepage = "https://github.com/SamoraDC/tetrad"
documentation = "https://docs.rs/tetrad"
readme = "README.md"
keywords = ["mcp", "claude", "code-review", "ai", "consensus"]
categories = ["development-tools", "command-line-utilities"]

[lib]
name = "tetrad"
path = "src/lib.rs"

[[bin]]
name = "tetrad"
path = "src/main.rs"

[features]
default = ["cli", "sqlite"]
cli = ["clap", "dialoguer", "indicatif"]
sqlite = ["rusqlite"]
postgres = ["sqlx"]  # For enterprise
plugins = ["libloading"]

[dependencies]
# Async runtime
tokio = { version = "1.45", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# MCP Protocol
# Note: use own implementation or community crate

# CLI
clap = { version = "4.5", features = ["derive"], optional = true }
dialoguer = { version = "0.11", optional = true }
indicatif = { version = "0.17", optional = true }

# Database
rusqlite = { version = "0.32", features = ["bundled"], optional = true }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"], optional = true }

# CLI execution
tokio-process = "0.2"

# Caching
lru = "0.12"

# Hashing
sha2 = "0.10"
hex = "0.4"

# Configuration
config = "0.14"
toml = "0.8"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Utilities
uuid = { version = "1.11", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
regex = "1.11"

# Plugins (optional)
libloading = { version = "0.8", optional = true }

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.13"
tempfile = "3.14"
assert_cmd = "2.0"
predicates = "3.1"

[profile.release]
lto = true
codegen-units = 1
strip = true

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

---

## 9. Publishing and Distribution

### 9.1 Publish to crates.io

```bash
# Login to crates.io
cargo login

# Verify before publishing
cargo publish --dry-run

# Publish
cargo publish
```

### 9.2 GitHub Releases with Binaries

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Package
        shell: bash
        run: |
          cd target/${{ matrix.target }}/release
          if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
            7z a ../../../tetrad-${{ matrix.target }}.zip tetrad.exe
          else
            tar czf ../../../tetrad-${{ matrix.target }}.tar.gz tetrad
          fi

      - name: Upload
        uses: softprops/action-gh-release@v1
        with:
          files: tetrad-*
```

### 9.3 Homebrew Formula

```ruby
# Formula/tetrad.rb
class Tetrad < Formula
  desc "Quadruple Consensus MCP for Claude Code"
  homepage "https://github.com/SamoraDC/tetrad"
  url "https://github.com/SamoraDC/tetrad/releases/download/v2.0.0/tetrad-x86_64-apple-darwin.tar.gz"
  sha256 "..."
  license "MIT"

  def install
    bin.install "tetrad"
  end

  test do
    assert_match "tetrad", shell_output("#{bin}/tetrad --version")
  end
end
```

---

## 10. Configuration in Claude Code

### 10.1 Automatic Addition (Recommended)

```bash
# Available in all projects
claude mcp add --scope user tetrad -- tetrad serve

# Or for current project only
claude mcp add tetrad -- tetrad serve
```

### 10.2 Manual in ~/.config/claude-code/settings.json

```json
{
  "mcpServers": {
    "tetrad": {
      "type": "stdio",
      "command": "tetrad",
      "args": ["serve"],
      "env": {
        "GEMINI_API_KEY": "${GEMINI_API_KEY}",
        "OPENAI_API_KEY": "${OPENAI_API_KEY}",
        "TETRAD_LOG_LEVEL": "info"
      }
    }
  }
}
```

---

## 11. Comparison with Claude-Flow

| Feature               | Claude-Flow               | Tetrad                        |
| --------------------- | ------------------------- | ----------------------------- |
| **Language**          | TypeScript                | Rust                          |
| **Focus**             | Agent orchestration       | Code validation               |
| **Learning**          | ReasoningBank (RuVector)  | ReasoningBank (SQLite)        |
| **Models**            | Claude/GPT/Gemini/Ollama  | Codex CLI/Gemini CLI/Qwen CLI |
| **Agents**            | 54+ agents                | 3 specialized evaluators      |
| **Consensus**         | Raft/Byzantine/Gossip     | Golden Rule/Weak/Strong       |
| **Installation**      | npm install               | cargo install                 |
| **MCP Tools**         | 175+                      | 6 focused                     |
| **Memory Usage**      | Medium (Node.js)          | Low (Rust)                    |
| **Latency**           | ~100ms                    | ~50ms                         |

### Using Together

Tetrad and Claude-Flow can work together:

```yaml
# claude-flow workflow that uses Tetrad
name: validated_swarm
triggers:
  - on_code_generated

steps:
  - name: validate_with_tetrad
    tool: tetrad_review_code
    on_block: abort_swarm

  - name: continue_swarm
    when: "{{ tetrad_result.consensus_achieved }}"
    action: proceed
```

---

## 12. Implementation Roadmap (Updated)

### Phase 1: Foundation ✅ COMPLETE

- [X] Setup Rust project with publishable crate structure
- [X] Basic CLI with clap (init, serve, status, config, doctor, version)
- [X] Implement CliExecutor trait
- [X] Basic unit tests (12 tests passing)
- [X] CodexExecutor, GeminiExecutor, QwenExecutor implemented
- [X] Health checks with `is_available()` and `version()`
- [X] Robust JSON parsing with `ExecutorResponse::parse_from_output()`

### Phase 2: Executors ✅ COMPLETE (included in Phase 1)

- [X] CodexExecutor
- [X] GeminiExecutor
- [X] QwenExecutor
- [X] Health checks
- [X] Robust JSON parsing

### Phase 3: Consensus + ReasoningBank ✅ COMPLETE

- [X] Consensus engine (rules.rs, aggregator.rs, engine.rs)
- [X] ReasoningBank with SQLite (bank.rs)
- [X] RETRIEVE→JUDGE→DISTILL→CONSOLIDATE cycle
- [X] Export/Import of patterns (export.rs)
- [X] Pattern matching (patterns.rs)
- [X] CLI commands: evaluate, history, export, import
- [X] 66 tests passing

### Phase 4: MCP Server ✅ COMPLETE

- [X] MCP protocol (stdio) - JSON-RPC 2.0 with Content-Length headers
- [X] 6 exposed tools (review_plan/code/tests, confirm, final_check, status)
- [X] LRU cache with TTL for evaluation results
- [X] Basic hooks (pre/post_evaluate, on_consensus, on_block)
- [X] Integrated confirmation system (confirm → final_check)
- [X] 126 tests passing

### Phase 5: Polish ✅ COMPLETE

- [X] Complete interactive CLI (dialoguer for config)
- [X] Documentation (README.md, CLAUDE.md, CHANGELOG.md)
- [X] Integration tests (219 tests passing: 141 unit + 78 integration)
- [X] GitHub Actions CI/CD (ci.yml, release.yml)
- [X] Fix executor args (positional prompt)
- [X] CacheConfig connected to system

### Phase 6: Release 🔄 IN PROGRESS

- [X] Publish to crates.io
- [ ] GitHub Releases with binaries
- [ ] Homebrew formula
- [ ] Announcement

---

## 13. Conclusion

**Tetrad v2.0** combines the best of previous plans with Claude-Flow innovations:

1. **Quadruple consensus**: 4 models must agree
2. **ReasoningBank**: Learns from each evaluation (RETRIEVE→JUDGE→DISTILL→CONSOLIDATE)
3. **Easy distribution**: `cargo install tetrad`
4. **Complete CLI**: Intuitive commands like Claude-Flow
5. **Extensible**: Hook and plugin system
6. **Cross-session**: Persistence with SQLite
7. **Shareable**: Export/Import of learned patterns

The system is ready to be used by any developer who wants **code validated by 4 intelligences** with **continuous learning**.
