//! Export utilities for CSV and JSON downloads

use chrono::{DateTime, Utc};
use serde::Serialize;
use wasm_bindgen::JsCast;
use web_sys::{Blob, HtmlAnchorElement, Url};

/// Generate current timestamp for filenames (YYYYMMDD-HHMMSS format)
fn timestamp() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%Y%m%d-%H%M%S").to_string()
}

/// Export data as CSV and trigger browser download
///
/// # Arguments
/// * `headers` - Column headers
/// * `rows` - Data rows (each row is a vector of strings)
/// * `filename` - Base filename (timestamp will be added)
///
/// # Example
/// ```ignore
/// export_as_csv(
///     vec!["Date", "Project", "Tokens"],
///     vec![
///         vec!["2024-01-01", "ccboard", "1000"],
///         vec!["2024-01-02", "rtk", "2000"],
///     ],
///     "sessions"
/// );
/// // Downloads: sessions-20240101-120000.csv
/// ```
pub fn export_as_csv(headers: Vec<String>, rows: Vec<Vec<String>>, filename: &str) {
    let mut csv = String::new();

    // UTF-8 BOM for Excel compatibility
    csv.push_str("\u{FEFF}");

    // Header row
    csv.push_str(&headers.join(","));
    csv.push('\n');

    // Data rows with proper CSV escaping
    for row in rows {
        let escaped: Vec<_> = row
            .iter()
            .map(|cell| {
                // Escape quotes and wrap in quotes if contains comma, quote, or newline
                if cell.contains(',') || cell.contains('"') || cell.contains('\n') {
                    format!("\"{}\"", cell.replace("\"", "\"\""))
                } else {
                    cell.clone()
                }
            })
            .collect();
        csv.push_str(&escaped.join(","));
        csv.push('\n');
    }

    // Trigger download
    let filename_with_ts = format!("{}-{}.csv", filename, timestamp());
    trigger_download(&csv, &filename_with_ts, "text/csv");
}

/// Export data as JSON and trigger browser download
///
/// # Arguments
/// * `data` - Any serializable data structure
/// * `filename` - Base filename (timestamp will be added)
///
/// # Example
/// ```ignore
/// #[derive(Serialize)]
/// struct Stats {
///     total_sessions: u32,
///     total_tokens: u64,
/// }
///
/// let stats = Stats { total_sessions: 42, total_tokens: 1000000 };
/// export_as_json(&stats, "stats");
/// // Downloads: stats-20240101-120000.json
/// ```
pub fn export_as_json<T: Serialize>(data: &T, filename: &str) {
    // Serialize with pretty printing
    let json = match serde_json::to_string_pretty(data) {
        Ok(j) => j,
        Err(e) => {
            tracing::error!("Failed to serialize JSON for export: {}", e);
            return;
        }
    };

    // Trigger download
    let filename_with_ts = format!("{}-{}.json", filename, timestamp());
    trigger_download(&json, &filename_with_ts, "application/json");
}

/// Trigger browser download via Blob and temporary anchor element
fn trigger_download(content: &str, filename: &str, mime_type: &str) {
    // Get window and document
    let window = match web_sys::window() {
        Some(w) => w,
        None => {
            tracing::error!("Failed to get window object");
            return;
        }
    };

    let document = match window.document() {
        Some(d) => d,
        None => {
            tracing::error!("Failed to get document object");
            return;
        }
    };

    // Create Blob
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&wasm_bindgen::JsValue::from_str(content));

    let blob_options = web_sys::BlobPropertyBag::new();
    blob_options.set_type(mime_type);

    let blob = match Blob::new_with_str_sequence_and_options(&blob_parts, &blob_options) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("Failed to create Blob: {:?}", e);
            return;
        }
    };

    // Create object URL
    let url = match Url::create_object_url_with_blob(&blob) {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Failed to create object URL: {:?}", e);
            return;
        }
    };

    // Create temporary anchor element and trigger click
    let anchor = match document
        .create_element("a")
        .ok()
        .and_then(|el| el.dyn_into::<HtmlAnchorElement>().ok())
    {
        Some(a) => a,
        None => {
            tracing::error!("Failed to create anchor element");
            let _ = Url::revoke_object_url(&url);
            return;
        }
    };

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    // Clean up object URL
    if let Err(e) = Url::revoke_object_url(&url) {
        tracing::error!("Failed to revoke object URL: {:?}", e);
    }
}
