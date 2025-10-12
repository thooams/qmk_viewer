use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CompatibilityStats {
    pub total_keyboards: usize,
    pub successful_parsing: usize,
    pub failed_parsing: usize,
    pub successful_ui_rendering: usize,
    pub failed_ui_rendering: usize,
    pub total_parse_time_ms: u64,
    pub total_render_time_ms: u64,
    pub error_breakdown: HashMap<String, usize>,
}

impl CompatibilityStats {
    pub fn new() -> Self {
        Self {
            total_keyboards: 0,
            successful_parsing: 0,
            failed_parsing: 0,
            successful_ui_rendering: 0,
            failed_ui_rendering: 0,
            total_parse_time_ms: 0,
            total_render_time_ms: 0,
            error_breakdown: HashMap::new(),
        }
    }
}

impl Default for CompatibilityStats {
    fn default() -> Self {
        Self::new()
    }

    pub fn parsing_success_rate(&self) -> f64 {
        if self.total_keyboards == 0 {
            0.0
        } else {
            (self.successful_parsing as f64 / self.total_keyboards as f64) * 100.0
        }
    }

    pub fn ui_rendering_success_rate(&self) -> f64 {
        if self.total_keyboards == 0 {
            0.0
        } else {
            (self.successful_ui_rendering as f64 / self.total_keyboards as f64) * 100.0
        }
    }

    pub fn avg_parse_time_ms(&self) -> f64 {
        if self.total_keyboards == 0 {
            0.0
        } else {
            self.total_parse_time_ms as f64 / self.total_keyboards as f64
        }
    }

    pub fn avg_render_time_ms(&self) -> f64 {
        if self.total_keyboards == 0 {
            0.0
        } else {
            self.total_render_time_ms as f64 / self.total_keyboards as f64
        }
    }
}

pub fn generate_comprehensive_report() -> Result<String, Box<dyn std::error::Error>> {
    let mut stats = CompatibilityStats::new();
    let mut successful_keyboards = Vec::new();
    let mut failed_keyboards = Vec::new();
    let mut ui_successful_keyboards = Vec::new();
    let mut ui_failed_keyboards = Vec::new();

    // Read parsing results
    if Path::new("tests/compatibility_report.md").exists() {
        let parsing_results = parse_compatibility_report("tests/compatibility_report.md")?;
        stats.total_keyboards = parsing_results.len();
        stats.successful_parsing = parsing_results.iter().filter(|r| r.success).count();
        stats.failed_parsing = stats.total_keyboards - stats.successful_parsing;
        stats.total_parse_time_ms = parsing_results.iter().map(|r| r.parse_time_ms).sum();

        for result in parsing_results {
            if result.success {
                successful_keyboards.push(result.clone());
            } else {
                if let Some(error) = &result.error_message {
                    let error_type = categorize_error(error);
                    *stats.error_breakdown.entry(error_type).or_insert(0) += 1;
                }
                failed_keyboards.push(result);
            }
        }
    }

    // Read UI rendering results
    if Path::new("tests/ui_rendering_report.md").exists() {
        let ui_results = parse_ui_rendering_report("tests/ui_rendering_report.md")?;
        stats.successful_ui_rendering = ui_results
            .iter()
            .filter(|r| r.layout_creation_success)
            .count();
        stats.failed_ui_rendering = ui_results.len() - stats.successful_ui_rendering;
        stats.total_render_time_ms = ui_results.iter().map(|r| r.render_time_ms).sum();

        for result in ui_results {
            if result.layout_creation_success {
                ui_successful_keyboards.push(result);
            } else {
                ui_failed_keyboards.push(result);
            }
        }
    }

    // Generate comprehensive report
    let mut report = String::new();

    // Header
    report.push_str("# QMK Keyboard Viewer - Comprehensive Compatibility Report\n\n");
    report.push_str(&format!(
        "Generated on: {}\n\n",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Executive Summary
    report.push_str("## Executive Summary\n\n");
    report.push_str(&format!(
        "This report provides a comprehensive analysis of QMK Keyboard Viewer's compatibility with {} keyboards from the QMK firmware repository.\n\n",
        stats.total_keyboards
    ));

    report.push_str("### Key Metrics\n\n");
    report.push_str(&format!(
        "- **Parsing Success Rate**: {:.1}% ({} of {} keyboards)\n",
        stats.parsing_success_rate(),
        stats.successful_parsing,
        stats.total_keyboards
    ));
    report.push_str(&format!(
        "- **UI Rendering Success Rate**: {:.1}% ({} of {} keyboards)\n",
        stats.ui_rendering_success_rate(),
        stats.successful_ui_rendering,
        stats.total_keyboards
    ));
    report.push_str(&format!(
        "- **Average Parse Time**: {:.1}ms\n",
        stats.avg_parse_time_ms()
    ));
    report.push_str(&format!(
        "- **Average Render Time**: {:.1}ms\n\n",
        stats.avg_render_time_ms()
    ));

    // Performance Analysis
    report.push_str("## Performance Analysis\n\n");
    report.push_str("### Parse Performance\n\n");
    report.push_str("| Metric | Value |\n");
    report.push_str("|--------|-------|\n");
    report.push_str(&format!(
        "| Total Parse Time | {}ms |\n",
        stats.total_parse_time_ms
    ));
    report.push_str(&format!(
        "| Average Parse Time | {:.1}ms |\n",
        stats.avg_parse_time_ms()
    ));
    report.push_str(&format!(
        "| Fastest Parse | {}ms |\n",
        successful_keyboards
            .iter()
            .map(|r| r.parse_time_ms)
            .min()
            .unwrap_or(0)
    ));
    report.push_str(&format!(
        "| Slowest Parse | {}ms |\n\n",
        successful_keyboards
            .iter()
            .map(|r| r.parse_time_ms)
            .max()
            .unwrap_or(0)
    ));

    // Error Analysis
    if !stats.error_breakdown.is_empty() {
        report.push_str("## Error Analysis\n\n");
        report.push_str("### Error Categories\n\n");
        report.push_str("| Error Type | Count | Percentage |\n");
        report.push_str("|------------|-------|------------|\n");

        let mut sorted_errors: Vec<_> = stats.error_breakdown.iter().collect();
        sorted_errors.sort_by(|a, b| b.1.cmp(a.1));

        for (error_type, count) in sorted_errors {
            let percentage = (*count as f64 / stats.failed_parsing as f64) * 100.0;
            report.push_str(&format!(
                "| {} | {} | {:.1}% |\n",
                error_type, count, percentage
            ));
        }
        report.push('\n');
    }

    // Successful Keyboards
    if !successful_keyboards.is_empty() {
        report.push_str("## Successfully Parsed Keyboards\n\n");
        report.push_str("| Keyboard | Layers | Keys/Layer | Parse Time (ms) |\n");
        report.push_str("|----------|--------|------------|-----------------|\n");

        // Sort by parse time for better analysis
        let mut sorted_successful = successful_keyboards.clone();
        sorted_successful.sort_by(|a, b| a.parse_time_ms.cmp(&b.parse_time_ms));

        for result in sorted_successful.iter().take(50) {
            // Show top 50
            report.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                result.keyboard_name,
                result.layers_count.unwrap_or(0),
                result.keys_per_layer.unwrap_or(0),
                result.parse_time_ms
            ));
        }

        if successful_keyboards.len() > 50 {
            report.push_str(&format!(
                "| ... and {} more | ... | ... | ... |\n\n",
                successful_keyboards.len() - 50
            ));
        } else {
            report.push('\n');
        }
    }

    // Failed Keyboards
    if !failed_keyboards.is_empty() {
        report.push_str("## Failed Keyboards\n\n");
        report.push_str("| Keyboard | Error | Parse Time (ms) |\n");
        report.push_str("|----------|-------|-----------------|\n");

        for result in failed_keyboards.iter().take(20) {
            // Show first 20 failures
            let error = result.error_message.as_deref().unwrap_or("Unknown error");
            report.push_str(&format!(
                "| {} | {} | {} |\n",
                result.keyboard_name, error, result.parse_time_ms
            ));
        }

        if failed_keyboards.len() > 20 {
            report.push_str(&format!(
                "| ... and {} more | ... | ... |\n\n",
                failed_keyboards.len() - 20
            ));
        } else {
            report.push('\n');
        }
    }

    // UI Rendering Results
    if !ui_successful_keyboards.is_empty() || !ui_failed_keyboards.is_empty() {
        report.push_str("## UI Rendering Results\n\n");

        if !ui_successful_keyboards.is_empty() {
            report.push_str("### Successfully Rendered Keyboards\n\n");
            report.push_str("| Keyboard | Rows | Cols | Total Keys | Render Time (ms) |\n");
            report.push_str("|----------|------|------|------------|------------------|\n");

            for result in ui_successful_keyboards.iter().take(20) {
                report.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    result.keyboard_name,
                    result.rows.unwrap_or(0),
                    result.cols.unwrap_or(0),
                    result.total_keys.unwrap_or(0),
                    result.render_time_ms
                ));
            }
            report.push('\n');
        }

        if !ui_failed_keyboards.is_empty() {
            report.push_str("### Failed UI Rendering\n\n");
            report.push_str("| Keyboard | Error | Render Time (ms) |\n");
            report.push_str("|----------|-------|------------------|\n");

            for result in ui_failed_keyboards.iter().take(10) {
                let error = result.error_message.as_deref().unwrap_or("Unknown error");
                report.push_str(&format!(
                    "| {} | {} | {} |\n",
                    result.keyboard_name, error, result.render_time_ms
                ));
            }
            report.push('\n');
        }
    }

    // Recommendations
    report.push_str("## Recommendations\n\n");

    if stats.parsing_success_rate() < 80.0 {
        report.push_str("### Parsing Improvements\n\n");
        report.push_str("- Consider improving the keymap parser to handle more edge cases\n");
        report.push_str("- Add support for additional QMK keycode formats\n");
        report.push_str("- Implement better error recovery mechanisms\n\n");
    }

    if stats.ui_rendering_success_rate() < 80.0 {
        report.push_str("### UI Rendering Improvements\n\n");
        report.push_str("- Enhance keyboard dimension detection algorithms\n");
        report.push_str("- Add support for non-standard keyboard layouts\n");
        report.push_str("- Improve keycode translation robustness\n\n");
    }

    if stats.avg_parse_time_ms() > 100.0 {
        report.push_str("### Performance Optimizations\n\n");
        report.push_str("- Consider caching parsed keymaps\n");
        report.push_str("- Optimize string processing operations\n");
        report.push_str("- Implement parallel parsing for batch operations\n\n");
    }

    // Conclusion
    report.push_str("## Conclusion\n\n");
    report.push_str(&format!(
        "QMK Keyboard Viewer demonstrates {}% compatibility with the tested QMK keyboards. ",
        stats.parsing_success_rate() as i32
    ));

    if stats.parsing_success_rate() >= 90.0 {
        report.push_str("This is excellent compatibility and indicates the application is ready for production use with most QMK keyboards.\n\n");
    } else if stats.parsing_success_rate() >= 70.0 {
        report.push_str("This is good compatibility with room for improvement. Consider addressing the most common error types to increase compatibility.\n\n");
    } else {
        report.push_str("This indicates significant compatibility issues that should be addressed before production use.\n\n");
    }

    Ok(report)
}

fn categorize_error(error: &str) -> String {
    if error.contains("Parse error") {
        "Parse Error".to_string()
    } else if error.contains("File read error") {
        "File Read Error".to_string()
    } else if error.contains("No layers found") {
        "No Layers Found".to_string()
    } else if error.contains("Insufficient keys") {
        "Insufficient Keys".to_string()
    } else if error.contains("Could not determine") {
        "Dimension Detection Error".to_string()
    } else {
        "Other Error".to_string()
    }
}

#[derive(Debug, Clone)]
struct ParsingResult {
    keyboard_name: String,
    success: bool,
    error_message: Option<String>,
    layers_count: Option<usize>,
    keys_per_layer: Option<usize>,
    parse_time_ms: u64,
}

#[derive(Debug, Clone)]
struct UIRenderingResult {
    keyboard_name: String,
    layout_creation_success: bool,
    error_message: Option<String>,
    rows: Option<usize>,
    cols: Option<usize>,
    total_keys: Option<usize>,
    render_time_ms: u64,
}

fn parse_compatibility_report(
    path: &str,
) -> Result<Vec<ParsingResult>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut results = Vec::new();

    // Simple parsing of the markdown report
    let lines: Vec<&str> = content.lines().collect();
    let mut in_successful_section = false;
    let mut in_failed_section = false;

    for line in lines {
        let line = line.trim();

        if line.contains("## Successful Keyboards") {
            in_successful_section = true;
            in_failed_section = false;
            continue;
        } else if line.contains("## Failed Keyboards") {
            in_successful_section = false;
            in_failed_section = true;
            continue;
        } else if line.starts_with("##") {
            in_successful_section = false;
            in_failed_section = false;
            continue;
        }

        if line.starts_with('|') && !line.contains("---") {
            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 4 {
                let keyboard_name = parts[1].to_string();

                if in_successful_section {
                    let layers_count = parts[2].parse().ok();
                    let keys_per_layer = parts[3].parse().ok();
                    let parse_time_ms = parts[4].parse().unwrap_or(0);

                    results.push(ParsingResult {
                        keyboard_name,
                        success: true,
                        error_message: None,
                        layers_count,
                        keys_per_layer,
                        parse_time_ms,
                    });
                } else if in_failed_section {
                    let error_message = Some(parts[2].to_string());
                    let parse_time_ms = parts[3].parse().unwrap_or(0);

                    results.push(ParsingResult {
                        keyboard_name,
                        success: false,
                        error_message,
                        layers_count: None,
                        keys_per_layer: None,
                        parse_time_ms,
                    });
                }
            }
        }
    }

    Ok(results)
}

fn parse_ui_rendering_report(
    path: &str,
) -> Result<Vec<UIRenderingResult>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut results = Vec::new();

    // Simple parsing of the markdown report
    let lines: Vec<&str> = content.lines().collect();
    let mut in_successful_section = false;
    let mut in_failed_section = false;

    for line in lines {
        let line = line.trim();

        if line.contains("## Successful UI Rendering") {
            in_successful_section = true;
            in_failed_section = false;
            continue;
        } else if line.contains("## Failed UI Rendering") {
            in_successful_section = false;
            in_failed_section = true;
            continue;
        } else if line.starts_with("##") {
            in_successful_section = false;
            in_failed_section = false;
            continue;
        }

        if line.starts_with('|') && !line.contains("---") {
            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 4 {
                let keyboard_name = parts[1].to_string();

                if in_successful_section {
                    let rows = parts[2].parse().ok();
                    let cols = parts[3].parse().ok();
                    let total_keys = parts[4].parse().ok();
                    let render_time_ms = parts[5].parse().unwrap_or(0);

                    results.push(UIRenderingResult {
                        keyboard_name,
                        layout_creation_success: true,
                        error_message: None,
                        rows,
                        cols,
                        total_keys,
                        render_time_ms,
                    });
                } else if in_failed_section {
                    let error_message = Some(parts[2].to_string());
                    let render_time_ms = parts[3].parse().unwrap_or(0);

                    results.push(UIRenderingResult {
                        keyboard_name,
                        layout_creation_success: false,
                        error_message,
                        rows: None,
                        cols: None,
                        total_keys: None,
                        render_time_ms,
                    });
                }
            }
        }
    }

    Ok(results)
}

#[test]
fn test_generate_comprehensive_report() {
    // This test will only work if the individual reports exist
    if Path::new("tests/compatibility_report.md").exists()
        || Path::new("tests/ui_rendering_report.md").exists()
    {
        match generate_comprehensive_report() {
            Ok(report) => {
                println!(
                    "Generated comprehensive report ({} characters)",
                    report.len()
                );
                // Save the report
                fs::write("tests/comprehensive_compatibility_report.md", &report)
                    .expect("Failed to write comprehensive report");
                println!(
                    "Comprehensive report saved to: tests/comprehensive_compatibility_report.md"
                );
            }
            Err(e) => {
                println!("Failed to generate comprehensive report: {}", e);
            }
        }
    } else {
        println!("No individual reports found. Run compatibility tests first.");
    }
}
