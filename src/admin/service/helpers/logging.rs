//! Logging operations helper module
//!
//! Provides functions for log retrieval, filtering, export, and statistics.

use crate::admin::service::helpers::admin_defaults;
use crate::admin::service::types::{
    AdminError, LogEntries, LogEntry, LogExportFormat, LogFilter, LogStats,
};
use crate::infrastructure::logging::SharedLogBuffer;
use crate::infrastructure::service_helpers::IteratorHelpers;
use std::collections::HashMap;

/// Get filtered log entries from the log buffer
pub async fn get_logs(
    log_buffer: &SharedLogBuffer,
    filter: LogFilter,
) -> Result<LogEntries, AdminError> {
    let core_entries = log_buffer.get_all().await;

    // Transform entries with predicate for filtering
    let mut entries: Vec<LogEntry> = IteratorHelpers::filter_collect(
        core_entries.into_iter().map(|e| LogEntry {
            timestamp: e.timestamp,
            level: e.level,
            module: e.target.clone(),
            message: e.message,
            target: e.target,
            file: None,
            line: None,
        }),
        |e| {
            // Apply all filter conditions in one pass
            if let Some(level) = &filter.level {
                if e.level != *level {
                    return false;
                }
            }
            if let Some(module) = &filter.module {
                if e.module != *module {
                    return false;
                }
            }
            if let Some(message_contains) = &filter.message_contains {
                if !e.message.contains(message_contains) {
                    return false;
                }
            }
            if let Some(start_time) = filter.start_time {
                if e.timestamp < start_time {
                    return false;
                }
            }
            if let Some(end_time) = filter.end_time {
                if e.timestamp > end_time {
                    return false;
                }
            }
            true
        },
    );

    let total_count = entries.len() as u64;

    if let Some(limit) = filter.limit {
        entries.truncate(limit);
    }

    Ok(LogEntries {
        entries,
        total_count,
        has_more: false,
    })
}

/// Export logs to a file in the specified format
pub async fn export_logs(
    log_buffer: &SharedLogBuffer,
    filter: LogFilter,
    format: LogExportFormat,
) -> Result<String, AdminError> {
    let log_entries = get_logs(log_buffer, filter).await?;
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let extension = match format {
        LogExportFormat::Json => "json",
        LogExportFormat::Csv => "csv",
        LogExportFormat::PlainText => "log",
    };

    let export_dir = std::path::PathBuf::from(admin_defaults::DEFAULT_EXPORTS_DIR);
    std::fs::create_dir_all(&export_dir).map_err(|e| {
        AdminError::ConfigError(format!("Failed to create exports directory: {}", e))
    })?;

    let filename = format!("logs_export_{}.{}", timestamp, extension);
    let filepath = export_dir.join(&filename);

    let content = match format {
        LogExportFormat::Json => serde_json::to_string_pretty(&log_entries.entries)
            .map_err(|e| AdminError::ConfigError(format!("JSON serialization failed: {}", e)))?,
        LogExportFormat::Csv => {
            let mut csv = String::from("timestamp,level,module,target,message\n");
            for entry in &log_entries.entries {
                csv.push_str(&format!(
                    "{},{},{},{},\"{}\"\n",
                    entry.timestamp.to_rfc3339(),
                    entry.level,
                    entry.module,
                    entry.target,
                    entry.message.replace('"', "\"\"")
                ));
            }
            csv
        }
        LogExportFormat::PlainText => {
            let mut text = String::new();
            for entry in &log_entries.entries {
                text.push_str(&format!(
                    "[{}] {} [{}] {}\n",
                    entry.timestamp.to_rfc3339(),
                    entry.level,
                    entry.target,
                    entry.message
                ));
            }
            text
        }
    };

    std::fs::write(&filepath, content)
        .map_err(|e| AdminError::ConfigError(format!("Failed to write log export: {}", e)))?;

    tracing::info!(
        "Logs exported to file: {} ({} entries)",
        filepath.display(),
        log_entries.entries.len()
    );
    Ok(filepath.to_string_lossy().to_string())
}

/// Get statistics about log entries
pub async fn get_log_stats(log_buffer: &SharedLogBuffer) -> Result<LogStats, AdminError> {
    let all_entries = log_buffer.get_all().await;
    let mut entries_by_level = HashMap::new();
    let mut entries_by_module = HashMap::new();

    for entry in &all_entries {
        *entries_by_level.entry(entry.level.clone()).or_insert(0) += 1;
        *entries_by_module.entry(entry.target.clone()).or_insert(0) += 1;
    }

    Ok(LogStats {
        total_entries: all_entries.len() as u64,
        entries_by_level,
        entries_by_module,
        oldest_entry: all_entries.first().map(|e| e.timestamp),
        newest_entry: all_entries.last().map(|e| e.timestamp),
    })
}
