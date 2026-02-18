//! Export functionality for billing blocks, sessions, and conversations
//!
//! Provides simple, testable export with proper error handling.

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;

use crate::models::{
    BillingBlockManager, ConversationMessage, MessageRole, SessionMetadata, StatsCache,
};

/// Export billing blocks to CSV format matching TUI table display
///
/// CSV columns: Date, Block (UTC), Tokens, Sessions, Cost
/// Rows sorted by date/time (most recent first)
///
/// # Arguments
/// * `manager` - Reference to BillingBlockManager
/// * `path` - Destination file path (created/overwritten)
///
/// # Errors
/// Returns error if file creation or write operations fail
///
/// # Examples
///
/// ```no_run
/// use ccboard_core::models::BillingBlockManager;
/// use ccboard_core::export::export_billing_blocks_to_csv;
/// use std::path::Path;
///
/// let manager = BillingBlockManager::new();
/// let path = Path::new("billing-blocks.csv");
/// export_billing_blocks_to_csv(&manager, &path).unwrap();
/// ```
pub fn export_billing_blocks_to_csv(manager: &BillingBlockManager, path: &Path) -> Result<()> {
    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let file = File::create(path)
        .with_context(|| format!("Failed to create CSV file: {}", path.display()))?;

    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(writer, "Date,Block (UTC),Tokens,Sessions,Cost")
        .context("Failed to write CSV header")?;

    // Get blocks sorted ascending, then reverse for descending (most recent first)
    let mut blocks = manager.get_all_blocks();
    blocks.reverse(); // Most recent first

    // Write data rows
    for (block, usage) in blocks {
        writeln!(
            writer,
            "\"{}\",\"{}\",{},{},\"${:.3}\"",
            block.date.format("%Y-%m-%d"), // "2026-02-03"
            block.label(),                 // "10:00-14:59"
            usage.total_tokens(),
            usage.session_count,
            usage.total_cost
        )
        .with_context(|| format!("Failed to write row for block {:?}", block))?;
    }

    writer.flush().context("Failed to flush CSV writer")?;

    Ok(())
}

/// Export sessions to CSV format
///
/// CSV columns: Date, Time, Project, Session ID, Messages, Tokens, Models, Duration (min)
/// Rows sorted by date/time (most recent first in input)
///
/// # Arguments
/// * `sessions` - Slice of SessionMetadata to export
/// * `path` - Destination file path (created/overwritten)
///
/// # Errors
/// Returns error if file creation or write operations fail
///
/// # Examples
///
/// ```no_run
/// use ccboard_core::models::SessionMetadata;
/// use ccboard_core::export::export_sessions_to_csv;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// let sessions: Vec<Arc<SessionMetadata>> = vec![]; // Load sessions
/// let path = Path::new("sessions.csv");
/// export_sessions_to_csv(&sessions, &path).unwrap();
/// ```
pub fn export_sessions_to_csv(sessions: &[Arc<SessionMetadata>], path: &Path) -> Result<()> {
    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let file = File::create(path)
        .with_context(|| format!("Failed to create CSV file: {}", path.display()))?;

    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(
        writer,
        "Date,Time,Project,Session ID,Messages,Tokens,Models,Duration (min)"
    )
    .context("Failed to write CSV header")?;

    // Write data rows
    for session in sessions {
        let date = session
            .first_timestamp
            .map(|ts| ts.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let time = session
            .first_timestamp
            .map(|ts| ts.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let models = session.models_used.join(";");

        let duration = if let (Some(first), Some(last)) =
            (&session.first_timestamp, &session.last_timestamp)
        {
            let diff = last.signed_duration_since(*first);
            diff.num_minutes()
        } else {
            0
        };

        writeln!(
            writer,
            "\"{}\",\"{}\",\"{}\",\"{}\",{},{},\"{}\",{}",
            date,
            time,
            session.project_path,
            session.id,
            session.message_count,
            session.total_tokens,
            models,
            duration
        )
        .with_context(|| format!("Failed to write row for session {}", session.id))?;
    }

    writer.flush().context("Failed to flush CSV writer")?;

    Ok(())
}

/// Export sessions to JSON format
///
/// Pretty-printed JSON array of session metadata
///
/// # Arguments
/// * `sessions` - Slice of SessionMetadata to export
/// * `path` - Destination file path (created/overwritten)
///
/// # Errors
/// Returns error if serialization or file write fails
///
/// # Examples
///
/// ```no_run
/// use ccboard_core::models::SessionMetadata;
/// use ccboard_core::export::export_sessions_to_json;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// let sessions: Vec<Arc<SessionMetadata>> = vec![]; // Load sessions
/// let path = Path::new("sessions.json");
/// export_sessions_to_json(&sessions, &path).unwrap();
/// ```
pub fn export_sessions_to_json(sessions: &[Arc<SessionMetadata>], path: &Path) -> Result<()> {
    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Dereference Arc to get &SessionMetadata for serialization
    let sessions_ref: Vec<&SessionMetadata> = sessions.iter().map(|s| s.as_ref()).collect();

    // Serialize to JSON (pretty print)
    let json = serde_json::to_string_pretty(&sessions_ref)
        .context("Failed to serialize sessions to JSON")?;

    // Write to file
    std::fs::write(path, json)
        .with_context(|| format!("Failed to write JSON file: {}", path.display()))?;

    Ok(())
}

// ============================================================================
// Stats Export Functions
// ============================================================================

/// Format a large number with K/M/B suffix for readability
fn fmt_num(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.2}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.2}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Export usage statistics to CSV (per-model breakdown)
///
/// CSV columns: Model, Input Tokens, Output Tokens, Cache Read, Cache Write, Total Tokens, Cost (USD)
/// Rows sorted by total token usage (highest first)
pub fn export_stats_to_csv(stats: &StatsCache, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let file = File::create(path)
        .with_context(|| format!("Failed to create CSV file: {}", path.display()))?;

    let mut writer = BufWriter::new(file);

    writeln!(
        writer,
        "Model,Input Tokens,Output Tokens,Cache Read,Cache Write,Total Tokens,Cost (USD)"
    )
    .context("Failed to write CSV header")?;

    let mut models: Vec<_> = stats
        .model_usage
        .iter()
        .filter(|(_, u)| u.total_tokens() > 0)
        .collect();
    models.sort_by(|a, b| b.1.total_tokens().cmp(&a.1.total_tokens()));

    for (name, usage) in &models {
        writeln!(
            writer,
            "\"{}\",{},{},{},{},{},\"{:.6}\"",
            name,
            usage.input_tokens,
            usage.output_tokens,
            usage.cache_read_input_tokens,
            usage.cache_creation_input_tokens,
            usage.total_tokens(),
            usage.cost_usd
        )
        .with_context(|| format!("Failed to write row for model {}", name))?;
    }

    writer.flush().context("Failed to flush CSV writer")?;

    Ok(())
}

/// Export usage statistics to JSON format (full StatsCache)
pub fn export_stats_to_json(stats: &StatsCache, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let json = serde_json::to_string_pretty(stats).context("Failed to serialize stats to JSON")?;

    std::fs::write(path, json)
        .with_context(|| format!("Failed to write JSON file: {}", path.display()))?;

    Ok(())
}

/// Export usage statistics to Markdown report
///
/// Generates a human-readable report with:
/// - Summary totals (tokens, sessions, messages, cache ratio)
/// - Per-model breakdown table
/// - Daily activity for last 30 days
pub fn export_stats_to_markdown(stats: &StatsCache, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let file = File::create(path)
        .with_context(|| format!("Failed to create Markdown file: {}", path.display()))?;

    let mut writer = BufWriter::new(file);

    writeln!(writer, "# Claude Code Statistics Report")?;
    writeln!(writer)?;

    if let Some(date) = &stats.last_computed_date {
        writeln!(writer, "**Generated**: {}", date)?;
    }
    if let Some(first) = &stats.first_session_date {
        writeln!(writer, "**First Session**: {}", first)?;
    }
    writeln!(writer)?;

    // Summary table
    writeln!(writer, "## Summary")?;
    writeln!(writer)?;
    writeln!(writer, "| Metric | Value |")?;
    writeln!(writer, "|--------|-------|")?;
    writeln!(
        writer,
        "| Total Tokens | {} |",
        fmt_num(stats.total_tokens())
    )?;
    writeln!(
        writer,
        "| Input Tokens | {} |",
        fmt_num(stats.total_input_tokens())
    )?;
    writeln!(
        writer,
        "| Output Tokens | {} |",
        fmt_num(stats.total_output_tokens())
    )?;
    writeln!(
        writer,
        "| Cache Read Tokens | {} |",
        fmt_num(stats.total_cache_read_tokens())
    )?;
    writeln!(
        writer,
        "| Cache Write Tokens | {} |",
        fmt_num(stats.total_cache_write_tokens())
    )?;
    writeln!(writer, "| Sessions | {} |", stats.total_sessions)?;
    writeln!(writer, "| Messages | {} |", stats.total_messages)?;
    writeln!(
        writer,
        "| Cache Hit Ratio | {:.1}% |",
        stats.cache_ratio() * 100.0
    )?;
    writeln!(writer)?;

    // Model breakdown
    if !stats.model_usage.is_empty() {
        writeln!(writer, "## Model Breakdown")?;
        writeln!(writer)?;
        writeln!(
            writer,
            "| Model | Input | Output | Cache Read | Cache Write | Total | Cost |"
        )?;
        writeln!(
            writer,
            "|-------|-------|--------|------------|-------------|-------|------|"
        )?;

        let mut models: Vec<_> = stats
            .model_usage
            .iter()
            .filter(|(_, u)| u.total_tokens() > 0)
            .collect();
        models.sort_by(|a, b| b.1.total_tokens().cmp(&a.1.total_tokens()));

        for (name, usage) in &models {
            writeln!(
                writer,
                "| {} | {} | {} | {} | {} | {} | ${:.4} |",
                name,
                fmt_num(usage.input_tokens),
                fmt_num(usage.output_tokens),
                fmt_num(usage.cache_read_input_tokens),
                fmt_num(usage.cache_creation_input_tokens),
                fmt_num(usage.total_tokens()),
                usage.cost_usd
            )
            .with_context(|| format!("Failed to write row for model {}", name))?;
        }

        writeln!(writer)?;
    }

    // Daily activity (last 30 days, most recent first)
    let recent = stats.recent_daily(30);
    if !recent.is_empty() {
        writeln!(writer, "## Daily Activity (Last 30 Days)")?;
        writeln!(writer)?;
        writeln!(writer, "| Date | Sessions | Messages | Tool Calls |")?;
        writeln!(writer, "|------|----------|----------|------------|")?;

        for day in recent.iter().rev() {
            writeln!(
                writer,
                "| {} | {} | {} | {} |",
                day.date, day.session_count, day.message_count, day.tool_call_count
            )?;
        }

        writeln!(writer)?;
    }

    writer.flush().context("Failed to flush Markdown writer")?;

    Ok(())
}

// ============================================================================
// Sessions Markdown Export
// ============================================================================

/// Export sessions list to Markdown table
pub fn export_sessions_to_markdown(sessions: &[Arc<SessionMetadata>], path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let file = File::create(path)
        .with_context(|| format!("Failed to create Markdown file: {}", path.display()))?;

    let mut writer = BufWriter::new(file);

    writeln!(writer, "# Session List")?;
    writeln!(writer)?;
    writeln!(writer, "**Total**: {} sessions", sessions.len())?;
    writeln!(writer)?;
    writeln!(
        writer,
        "| Date | Time | Project | Session ID | Messages | Tokens | Duration |"
    )?;
    writeln!(
        writer,
        "|------|------|---------|------------|----------|--------|----------|"
    )?;

    for session in sessions {
        let date = session
            .first_timestamp
            .map(|ts| ts.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let time = session
            .first_timestamp
            .map(|ts| ts.format("%H:%M").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let duration = if let (Some(first), Some(last)) =
            (&session.first_timestamp, &session.last_timestamp)
        {
            let diff = last.signed_duration_since(*first);
            format!("{}min", diff.num_minutes())
        } else {
            "N/A".to_string()
        };

        let short_id = &session.id[..8.min(session.id.len())];

        writeln!(
            writer,
            "| {} | {} | {} | `{}` | {} | {} | {} |",
            date,
            time,
            session.project_path,
            short_id,
            session.message_count,
            fmt_num(session.total_tokens),
            duration
        )
        .with_context(|| format!("Failed to write row for session {}", session.id))?;
    }

    writer.flush().context("Failed to flush Markdown writer")?;

    Ok(())
}

// ============================================================================
// Billing Blocks Additional Export Formats
// ============================================================================

/// Export billing blocks to JSON format
pub fn export_billing_blocks_to_json(manager: &BillingBlockManager, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let mut blocks = manager.get_all_blocks();
    blocks.reverse(); // Most recent first

    let json_array: Vec<_> = blocks
        .iter()
        .map(|(block, usage)| {
            serde_json::json!({
                "date": block.date.format("%Y-%m-%d").to_string(),
                "block": block.label(),
                "input_tokens": usage.input_tokens,
                "output_tokens": usage.output_tokens,
                "cache_creation_tokens": usage.cache_creation_tokens,
                "cache_read_tokens": usage.cache_read_tokens,
                "total_tokens": usage.total_tokens(),
                "sessions": usage.session_count,
                "cost_usd": usage.total_cost,
            })
        })
        .collect();

    let json = serde_json::to_string_pretty(&json_array)
        .context("Failed to serialize billing blocks to JSON")?;

    std::fs::write(path, json)
        .with_context(|| format!("Failed to write JSON file: {}", path.display()))?;

    Ok(())
}

/// Export billing blocks to Markdown table
pub fn export_billing_blocks_to_markdown(manager: &BillingBlockManager, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let file = File::create(path)
        .with_context(|| format!("Failed to create Markdown file: {}", path.display()))?;

    let mut writer = BufWriter::new(file);

    writeln!(writer, "# Billing Blocks Report")?;
    writeln!(writer)?;
    writeln!(writer, "| Date | Block (UTC) | Tokens | Sessions | Cost |")?;
    writeln!(writer, "|------|-------------|--------|----------|------|")?;

    let mut blocks = manager.get_all_blocks();
    blocks.reverse(); // Most recent first

    for (block, usage) in &blocks {
        writeln!(
            writer,
            "| {} | {} | {} | {} | ${:.3} |",
            block.date.format("%Y-%m-%d"),
            block.label(),
            fmt_num(usage.total_tokens()),
            usage.session_count,
            usage.total_cost
        )
        .with_context(|| format!("Failed to write row for block {:?}", block))?;
    }

    writer.flush().context("Failed to flush Markdown writer")?;

    Ok(())
}

// ============================================================================
// Conversation Export Functions
// ============================================================================

/// Export conversation to Markdown format
///
/// Format:
/// ```markdown
/// # Session: abc123
/// **Project**: /path/to/project
/// **Date**: 2026-02-12 14:30:00
/// **Messages**: 42 | **Tokens**: 15000
///
/// ## User
/// Can you help me with...
///
/// ## Assistant (claude-sonnet-4-5)
/// Sure! Here's how...
/// ```
///
/// # Arguments
/// * `messages` - Conversation messages to export
/// * `metadata` - Session metadata (ID, project, timestamps)
/// * `path` - Destination file path
///
/// # Errors
/// Returns error if file creation or write operations fail
pub fn export_conversation_to_markdown(
    messages: &[ConversationMessage],
    metadata: &SessionMetadata,
    path: &Path,
) -> Result<()> {
    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let file = File::create(path)
        .with_context(|| format!("Failed to create Markdown file: {}", path.display()))?;

    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(writer, "# Session: {}", metadata.id)?;
    writeln!(writer, "**Project**: {}", metadata.project_path)?;

    if let Some(ts) = metadata.first_timestamp {
        writeln!(writer, "**Date**: {}", ts.format("%Y-%m-%d %H:%M:%S"))?;
    }

    writeln!(
        writer,
        "**Messages**: {} | **Tokens**: {}\n",
        metadata.message_count, metadata.total_tokens
    )?;

    // Write messages
    for msg in messages {
        let role = match msg.role {
            MessageRole::User => "User".to_string(),
            MessageRole::Assistant => {
                if let Some(ref model) = msg.model {
                    format!("Assistant ({})", model)
                } else {
                    "Assistant".to_string()
                }
            }
            MessageRole::System => "System".to_string(),
        };

        writeln!(writer, "## {}\n", role)?;

        // Write message content with proper escaping
        writeln!(writer, "{}\n", msg.content)?;

        // Optionally add token info for assistant messages
        if msg.role == MessageRole::Assistant {
            if let Some(ref tokens) = msg.tokens {
                writeln!(
                    writer,
                    "*Tokens: {} input, {} output*\n",
                    tokens.input_tokens, tokens.output_tokens
                )?;
            }
        }
    }

    writer.flush().context("Failed to flush Markdown writer")?;

    Ok(())
}

/// Export conversation to JSON format
///
/// Format:
/// ```json
/// {
///   "session_id": "abc123",
///   "project_path": "/path/to/project",
///   "metadata": {...},
///   "messages": [
///     {
///       "role": "user",
///       "content": "...",
///       "timestamp": "2026-02-12T14:30:00Z",
///       "model": null,
///       "tokens": null
///     }
///   ]
/// }
/// ```
///
/// # Arguments
/// * `messages` - Conversation messages to export
/// * `metadata` - Session metadata
/// * `path` - Destination file path
///
/// # Errors
/// Returns error if serialization or file write fails
pub fn export_conversation_to_json(
    messages: &[ConversationMessage],
    metadata: &SessionMetadata,
    path: &Path,
) -> Result<()> {
    use serde_json::json;

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Build structured JSON
    let export_data = json!({
        "session_id": metadata.id,
        "project_path": metadata.project_path,
        "metadata": {
            "first_timestamp": metadata.first_timestamp,
            "last_timestamp": metadata.last_timestamp,
            "message_count": metadata.message_count,
            "total_tokens": metadata.total_tokens,
            "models_used": metadata.models_used,
            "duration_seconds": metadata.duration_seconds,
            "branch": metadata.branch,
        },
        "messages": messages,
    });

    // Serialize to JSON (pretty print)
    let json_str =
        serde_json::to_string_pretty(&export_data).context("Failed to serialize conversation")?;

    // Write to file
    std::fs::write(path, json_str)
        .with_context(|| format!("Failed to write JSON file: {}", path.display()))?;

    Ok(())
}

/// Export conversation to HTML format
///
/// Generates a styled HTML report with:
/// - Session metadata header
/// - Messages with role-based styling
/// - Token usage info
/// - Responsive design
///
/// # Arguments
/// * `messages` - Conversation messages to export
/// * `metadata` - Session metadata
/// * `path` - Destination file path
///
/// # Errors
/// Returns error if file creation or write operations fail
pub fn export_conversation_to_html(
    messages: &[ConversationMessage],
    metadata: &SessionMetadata,
    path: &Path,
) -> Result<()> {
    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let file = File::create(path)
        .with_context(|| format!("Failed to create HTML file: {}", path.display()))?;

    let mut writer = BufWriter::new(file);

    // Write HTML header
    writeln!(writer, "<!DOCTYPE html>")?;
    writeln!(writer, "<html lang=\"en\">")?;
    writeln!(writer, "<head>")?;
    writeln!(writer, "    <meta charset=\"UTF-8\">")?;
    writeln!(
        writer,
        "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">"
    )?;
    writeln!(
        writer,
        "    <title>Session {} - ccboard</title>",
        metadata.id
    )?;
    writeln!(writer, "    <style>")?;
    writeln!(
        writer,
        "        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 900px; margin: 40px auto; padding: 20px; background: #f5f5f5; }}"
    )?;
    writeln!(
        writer,
        "        .header {{ background: white; padding: 30px; border-radius: 8px; margin-bottom: 30px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); }}"
    )?;
    writeln!(
        writer,
        "        .header h1 {{ margin: 0 0 20px 0; color: #333; font-size: 28px; }}"
    )?;
    writeln!(writer, "        .meta {{ color: #666; line-height: 1.8; }}")?;
    writeln!(
        writer,
        "        .message {{ background: white; padding: 20px; margin-bottom: 15px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}"
    )?;
    writeln!(
        writer,
        "        .role {{ font-weight: 600; margin-bottom: 10px; padding: 8px 12px; border-radius: 4px; display: inline-block; }}"
    )?;
    writeln!(
        writer,
        "        .role.user {{ background: #e3f2fd; color: #1976d2; }}"
    )?;
    writeln!(
        writer,
        "        .role.assistant {{ background: #f3e5f5; color: #7b1fa2; }}"
    )?;
    writeln!(
        writer,
        "        .role.system {{ background: #fff3e0; color: #e65100; }}"
    )?;
    writeln!(
        writer,
        "        .content {{ white-space: pre-wrap; line-height: 1.6; color: #333; }}"
    )?;
    writeln!(
        writer,
        "        .tokens {{ color: #999; font-size: 12px; margin-top: 10px; }}"
    )?;
    writeln!(writer, "    </style>")?;
    writeln!(writer, "</head>")?;
    writeln!(writer, "<body>")?;

    // Write session header
    writeln!(writer, "    <div class=\"header\">")?;
    writeln!(
        writer,
        "        <h1>Session {}</h1>",
        html_escape(&metadata.id)
    )?;
    writeln!(writer, "        <div class=\"meta\">")?;
    writeln!(
        writer,
        "            <strong>Project:</strong> {}<br>",
        html_escape(&metadata.project_path)
    )?;

    if let Some(ts) = metadata.first_timestamp {
        writeln!(
            writer,
            "            <strong>Date:</strong> {}<br>",
            ts.format("%Y-%m-%d %H:%M:%S")
        )?;
    }

    writeln!(
        writer,
        "            <strong>Messages:</strong> {} | <strong>Tokens:</strong> {}",
        metadata.message_count, metadata.total_tokens
    )?;
    writeln!(writer, "        </div>")?;
    writeln!(writer, "    </div>")?;

    // Write messages
    for msg in messages {
        let role_class = match msg.role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
        };

        let role_label = match msg.role {
            MessageRole::User => "User".to_string(),
            MessageRole::Assistant => {
                if let Some(ref model) = msg.model {
                    format!("Assistant ({})", model)
                } else {
                    "Assistant".to_string()
                }
            }
            MessageRole::System => "System".to_string(),
        };

        writeln!(writer, "    <div class=\"message\">")?;
        writeln!(
            writer,
            "        <div class=\"role {}\">{}</div>",
            role_class,
            html_escape(&role_label)
        )?;
        writeln!(
            writer,
            "        <div class=\"content\">{}</div>",
            html_escape(&msg.content)
        )?;

        // Add token info for assistant messages
        if msg.role == MessageRole::Assistant {
            if let Some(ref tokens) = msg.tokens {
                writeln!(
                    writer,
                    "        <div class=\"tokens\">Tokens: {} input, {} output</div>",
                    tokens.input_tokens, tokens.output_tokens
                )?;
            }
        }

        writeln!(writer, "    </div>")?;
    }

    writeln!(writer, "</body>")?;
    writeln!(writer, "</html>")?;

    writer.flush().context("Failed to flush HTML writer")?;

    Ok(())
}

/// HTML escape for safe output
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BillingBlockManager;
    use chrono::{TimeZone, Utc};
    use tempfile::TempDir;

    #[test]
    fn test_export_empty_manager() {
        let manager = BillingBlockManager::new();
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("test.csv");

        export_billing_blocks_to_csv(&manager, &csv_path).unwrap();

        let contents = std::fs::read_to_string(&csv_path).unwrap();
        assert_eq!(contents, "Date,Block (UTC),Tokens,Sessions,Cost\n");
    }

    #[test]
    fn test_export_with_data() {
        let mut manager = BillingBlockManager::new();

        // Add sample data (2 blocks)
        let ts1 = Utc.with_ymd_and_hms(2026, 2, 3, 14, 30, 0).unwrap();
        manager.add_usage(&ts1, 5000, 1500, 200, 100, 0.015);

        let ts2 = Utc.with_ymd_and_hms(2026, 2, 3, 20, 15, 0).unwrap();
        manager.add_usage(&ts2, 3000, 1000, 100, 50, 0.010);

        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("billing.csv");

        export_billing_blocks_to_csv(&manager, &csv_path).unwrap();

        let contents = std::fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();

        assert_eq!(lines.len(), 3); // Header + 2 blocks
        assert_eq!(lines[0], "Date,Block (UTC),Tokens,Sessions,Cost");
        assert!(lines[1].contains("2026-02-03"));
        assert!(lines[1].contains("20:00-23:59")); // Later block first (reversed)
        assert!(lines[2].contains("10:00-14:59"));
    }

    #[test]
    fn test_creates_parent_directory() {
        let manager = BillingBlockManager::new();
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("exports/nested/test.csv");

        export_billing_blocks_to_csv(&manager, &nested_path).unwrap();

        assert!(nested_path.exists());
    }

    #[test]
    fn test_cost_formatting() {
        let mut manager = BillingBlockManager::new();

        // Test various cost values to verify 3 decimal places
        let ts = Utc.with_ymd_and_hms(2026, 2, 3, 10, 0, 0).unwrap();
        manager.add_usage(&ts, 1000, 500, 50, 25, 1.23456); // Should be $1.235

        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("cost.csv");

        export_billing_blocks_to_csv(&manager, &csv_path).unwrap();

        let contents = std::fs::read_to_string(&csv_path).unwrap();
        assert!(contents.contains("\"$1.235\""));
    }

    #[test]
    fn test_multiple_dates_sorted() {
        let mut manager = BillingBlockManager::new();

        // Add blocks for multiple dates
        let ts1 = Utc.with_ymd_and_hms(2026, 2, 1, 10, 0, 0).unwrap();
        manager.add_usage(&ts1, 1000, 500, 0, 0, 0.5);

        let ts2 = Utc.with_ymd_and_hms(2026, 2, 3, 5, 0, 0).unwrap();
        manager.add_usage(&ts2, 2000, 1000, 0, 0, 1.0);

        let ts3 = Utc.with_ymd_and_hms(2026, 2, 2, 15, 0, 0).unwrap();
        manager.add_usage(&ts3, 1500, 750, 0, 0, 0.75);

        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("sorted.csv");

        export_billing_blocks_to_csv(&manager, &csv_path).unwrap();

        let contents = std::fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();

        assert_eq!(lines.len(), 4); // Header + 3 blocks
                                    // Most recent first
        assert!(lines[1].contains("2026-02-03")); // Feb 3
        assert!(lines[2].contains("2026-02-02")); // Feb 2
        assert!(lines[3].contains("2026-02-01")); // Feb 1
    }

    // Session export tests
    use crate::models::SessionMetadata;
    use std::path::PathBuf;

    fn create_test_session(id: &str, project: &str, messages: u64, tokens: u64) -> SessionMetadata {
        let timestamp = Utc.with_ymd_and_hms(2026, 2, 3, 14, 30, 0).unwrap();
        SessionMetadata {
            id: id.into(),
            file_path: PathBuf::from(format!("/test/{}.jsonl", id)),
            project_path: project.into(),
            first_timestamp: Some(timestamp),
            last_timestamp: Some(timestamp + chrono::Duration::minutes(45)),
            message_count: messages,
            total_tokens: tokens,
            input_tokens: tokens / 2,
            output_tokens: tokens / 3,
            cache_creation_tokens: tokens / 10,
            cache_read_tokens: tokens - (tokens / 2 + tokens / 3 + tokens / 10),
            models_used: vec!["sonnet".to_string(), "opus".to_string()],
            file_size_bytes: 1024,
            first_user_message: Some("Test message".to_string()),
            has_subagents: false,
            duration_seconds: Some(2700), // 45 minutes
            branch: None,
            tool_usage: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_export_sessions_csv_empty() {
        let sessions: Vec<Arc<SessionMetadata>> = vec![];
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("sessions.csv");

        super::export_sessions_to_csv(&sessions, &csv_path).unwrap();

        let contents = std::fs::read_to_string(&csv_path).unwrap();
        assert_eq!(
            contents,
            "Date,Time,Project,Session ID,Messages,Tokens,Models,Duration (min)\n"
        );
    }

    #[test]
    fn test_export_sessions_csv_with_data() {
        let sessions = vec![
            Arc::new(create_test_session(
                "abc123",
                "/Users/test/project1",
                25,
                15000,
            )),
            Arc::new(create_test_session(
                "def456",
                "/Users/test/project2",
                10,
                5000,
            )),
        ];

        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("sessions.csv");

        super::export_sessions_to_csv(&sessions, &csv_path).unwrap();

        let contents = std::fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();

        assert_eq!(lines.len(), 3); // Header + 2 sessions
        assert_eq!(
            lines[0],
            "Date,Time,Project,Session ID,Messages,Tokens,Models,Duration (min)"
        );
        assert!(lines[1].contains("2026-02-03"));
        assert!(lines[1].contains("14:30:00"));
        assert!(lines[1].contains("abc123"));
        assert!(lines[1].contains("25"));
        assert!(lines[1].contains("15000"));
        assert!(lines[1].contains("sonnet;opus"));
        assert!(lines[1].contains("45")); // duration in minutes
    }

    #[test]
    fn test_export_sessions_json_empty() {
        let sessions: Vec<Arc<SessionMetadata>> = vec![];
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("sessions.json");

        super::export_sessions_to_json(&sessions, &json_path).unwrap();

        let contents = std::fs::read_to_string(&json_path).unwrap();
        assert_eq!(contents, "[]");
    }

    #[test]
    fn test_export_sessions_json_with_data() {
        let sessions = vec![Arc::new(create_test_session(
            "abc123",
            "/Users/test/project1",
            25,
            15000,
        ))];

        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("sessions.json");

        super::export_sessions_to_json(&sessions, &json_path).unwrap();

        let contents = std::fs::read_to_string(&json_path).unwrap();

        // Verify it's valid JSON
        let parsed: Vec<SessionMetadata> = serde_json::from_str(&contents).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].id, "abc123");
        assert_eq!(parsed[0].message_count, 25);
        assert_eq!(parsed[0].total_tokens, 15000);
    }

    #[test]
    fn test_export_sessions_creates_dirs() {
        let sessions = vec![Arc::new(create_test_session("test", "/test", 1, 100))];
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("exports/nested/sessions.csv");

        super::export_sessions_to_csv(&sessions, &nested_path).unwrap();

        assert!(nested_path.exists());
    }

    // Conversation export tests
    use crate::models::{ConversationMessage, MessageRole, TokenUsage};

    fn create_test_messages() -> Vec<ConversationMessage> {
        vec![
            ConversationMessage {
                role: MessageRole::User,
                content: "Hello, can you help me with Rust?".to_string(),
                timestamp: Some(Utc.with_ymd_and_hms(2026, 2, 12, 14, 30, 0).unwrap()),
                model: None,
                tokens: None,
                tool_calls: Vec::new(),
                tool_results: Vec::new(),
            },
            ConversationMessage {
                role: MessageRole::Assistant,
                content: "Sure! I'd be happy to help. What do you need?".to_string(),
                timestamp: Some(Utc.with_ymd_and_hms(2026, 2, 12, 14, 30, 30).unwrap()),
                model: Some("claude-sonnet-4-5-20250929".to_string()),
                tokens: Some(TokenUsage {
                    input_tokens: 100,
                    output_tokens: 50,
                    cache_read_tokens: 0,
                    cache_write_tokens: 0,
                }),
                tool_calls: Vec::new(),
                tool_results: Vec::new(),
            },
        ]
    }

    #[test]
    fn test_export_conversation_markdown() {
        let messages = create_test_messages();
        let metadata = create_test_session("test-conv", "/test/project", 2, 150);
        let temp_dir = TempDir::new().unwrap();
        let md_path = temp_dir.path().join("conversation.md");

        super::export_conversation_to_markdown(&messages, &metadata, &md_path).unwrap();

        let contents = std::fs::read_to_string(&md_path).unwrap();
        assert!(contents.contains("# Session: test-conv"));
        assert!(contents.contains("**Project**: /test/project"));
        assert!(contents.contains("## User"));
        assert!(contents.contains("Hello, can you help me with Rust?"));
        assert!(contents.contains("## Assistant (claude-sonnet-4-5-20250929)"));
        assert!(contents.contains("Sure! I'd be happy to help."));
        assert!(contents.contains("*Tokens: 100 input, 50 output*"));
    }

    #[test]
    fn test_export_conversation_json() {
        let messages = create_test_messages();
        let metadata = create_test_session("test-conv", "/test/project", 2, 150);
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("conversation.json");

        super::export_conversation_to_json(&messages, &metadata, &json_path).unwrap();

        let contents = std::fs::read_to_string(&json_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();

        assert_eq!(parsed["session_id"], "test-conv");
        assert_eq!(parsed["project_path"], "/test/project");
        assert_eq!(parsed["messages"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["messages"][0]["role"], "user");
        assert_eq!(parsed["messages"][1]["role"], "assistant");
    }

    #[test]
    fn test_export_conversation_html() {
        let messages = create_test_messages();
        let metadata = create_test_session("test-conv", "/test/project", 2, 150);
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("conversation.html");

        super::export_conversation_to_html(&messages, &metadata, &html_path).unwrap();

        let contents = std::fs::read_to_string(&html_path).unwrap();
        assert!(contents.contains("<!DOCTYPE html>"));
        assert!(contents.contains("<title>Session test-conv - ccboard</title>"));
        assert!(contents.contains("class=\"role user\""));
        assert!(contents.contains("class=\"role assistant\""));
        assert!(contents.contains("Hello, can you help me with Rust?"));
        assert!(contents.contains("Sure! I&#x27;d be happy to help."));
        assert!(contents.contains("Tokens: 100 input, 50 output"));
    }

    #[test]
    fn test_html_escape() {
        let input = "<script>alert('XSS')</script>";
        let escaped = super::html_escape(input);
        assert_eq!(
            escaped,
            "&lt;script&gt;alert(&#x27;XSS&#x27;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_export_creates_nested_dirs() {
        let messages = create_test_messages();
        let metadata = create_test_session("test", "/test", 2, 150);
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("exports/conversations/test.md");

        super::export_conversation_to_markdown(&messages, &metadata, &nested_path).unwrap();

        assert!(nested_path.exists());
    }
}
