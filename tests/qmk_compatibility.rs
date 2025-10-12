use std::fs;
use std::path::Path;
use std::collections::HashMap;
use qmk_viewer::keymap_c;

#[derive(Debug)]
struct CompatibilityResult {
    keyboard_name: String,
    success: bool,
    error_message: Option<String>,
    layers_count: Option<usize>,
    keys_per_layer: Option<usize>,
    parse_time_ms: u64,
}

impl CompatibilityResult {
    fn success(keyboard_name: String, layers_count: usize, keys_per_layer: usize, parse_time_ms: u64) -> Self {
        Self {
            keyboard_name,
            success: true,
            error_message: None,
            layers_count: Some(layers_count),
            keys_per_layer: Some(keys_per_layer),
            parse_time_ms,
        }
    }

    fn failure(keyboard_name: String, error_message: String, parse_time_ms: u64) -> Self {
        Self {
            keyboard_name,
            success: false,
            error_message: Some(error_message),
            layers_count: None,
            keys_per_layer: None,
            parse_time_ms,
        }
    }
}

fn read_keyboards_manifest() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let manifest_path = "tests/files/keyboards.txt";
    let content = fs::read_to_string(manifest_path)?;
    
    let keyboards: Vec<String> = content
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect();
    
    Ok(keyboards)
}

fn test_keymap_parsing(keyboard_name: &str) -> CompatibilityResult {
    let start_time = std::time::Instant::now();
    let file_path = format!("tests/files/{}", keyboard_name);
    
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            match keymap_c::parse_keymap_c(&content) {
                Ok(keymap) => {
                    let parse_time = start_time.elapsed().as_millis() as u64;
                    
                    // Extract keyboard name without _keymap.c suffix
                    let clean_name = keyboard_name.replace("_keymap.c", "");
                    
                    // Count layers and keys
                    let layers_count = keymap.layers.len();
                    let keys_per_layer = if layers_count > 0 {
                        keymap.layers[0].len()
                    } else {
                        0
                    };
                    
                    CompatibilityResult::success(
                        clean_name,
                        layers_count,
                        keys_per_layer,
                        parse_time
                    )
                }
                Err(e) => {
                    let parse_time = start_time.elapsed().as_millis() as u64;
                    let clean_name = keyboard_name.replace("_keymap.c", "");
                    CompatibilityResult::failure(
                        clean_name,
                        format!("Parse error: {}", e),
                        parse_time
                    )
                }
            }
        }
        Err(e) => {
            let parse_time = start_time.elapsed().as_millis() as u64;
            let clean_name = keyboard_name.replace("_keymap.c", "");
            CompatibilityResult::failure(
                clean_name,
                format!("File read error: {}", e),
                parse_time
            )
        }
    }
}

fn generate_compatibility_report(results: &[CompatibilityResult]) -> String {
    let total_tests = results.len();
    let successful_tests = results.iter().filter(|r| r.success).count();
    let failed_tests = total_tests - successful_tests;
    let success_rate = if total_tests > 0 {
        (successful_tests as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    let mut report = String::new();
    report.push_str("# QMK Keyboard Compatibility Report\n\n");
    report.push_str(&format!("Generated on: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    // Summary
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- **Total keyboards tested**: {}\n", total_tests));
    report.push_str(&format!("- **Successful**: {}\n", successful_tests));
    report.push_str(&format!("- **Failed**: {}\n", failed_tests));
    report.push_str(&format!("- **Success rate**: {:.1}%\n\n", success_rate));

    // Performance stats
    let total_parse_time: u64 = results.iter().map(|r| r.parse_time_ms).sum();
    let avg_parse_time = if total_tests > 0 {
        total_parse_time as f64 / total_tests as f64
    } else {
        0.0
    };
    
    report.push_str("## Performance\n\n");
    report.push_str(&format!("- **Total parse time**: {}ms\n", total_parse_time));
    report.push_str(&format!("- **Average parse time**: {:.1}ms\n\n", avg_parse_time));

    // Successful keyboards
    let successful: Vec<_> = results.iter().filter(|r| r.success).collect();
    if !successful.is_empty() {
        report.push_str("## Successful Keyboards\n\n");
        report.push_str("| Keyboard | Layers | Keys/Layer | Parse Time (ms) |\n");
        report.push_str("|----------|--------|------------|-----------------|\n");
        
        for result in &successful {
            report.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                result.keyboard_name,
                result.layers_count.unwrap_or(0),
                result.keys_per_layer.unwrap_or(0),
                result.parse_time_ms
            ));
        }
        report.push_str("\n");
    }

    // Failed keyboards
    let failed: Vec<_> = results.iter().filter(|r| !r.success).collect();
    if !failed.is_empty() {
        report.push_str("## Failed Keyboards\n\n");
        report.push_str("| Keyboard | Error | Parse Time (ms) |\n");
        report.push_str("|----------|-------|-----------------|\n");
        
        for result in &failed {
            let error = result.error_message.as_deref().unwrap_or("Unknown error");
            report.push_str(&format!(
                "| {} | {} | {} |\n",
                result.keyboard_name,
                error,
                result.parse_time_ms
            ));
        }
        report.push_str("\n");
    }

    // Error analysis
    if !failed.is_empty() {
        let mut error_counts: HashMap<String, usize> = HashMap::new();
        for result in &failed {
            if let Some(error) = &result.error_message {
                let error_type = if error.contains("Parse error") {
                    "Parse error"
                } else if error.contains("File read error") {
                    "File read error"
                } else {
                    "Other error"
                };
                *error_counts.entry(error_type.to_string()).or_insert(0) += 1;
            }
        }

        report.push_str("## Error Analysis\n\n");
        for (error_type, count) in error_counts {
            report.push_str(&format!("- **{}**: {} keyboards\n", error_type, count));
        }
        report.push_str("\n");
    }

    report
}

#[test]
fn test_all_qmk_keyboards() {
    println!("🔍 Starting QMK keyboard compatibility tests...");
    
    // Check if manifest exists
    if !Path::new("tests/files/keyboards.txt").exists() {
        panic!("❌ Keyboards manifest not found. Please run: ./scripts/collect-qmk-keymaps.sh");
    }

    // Read keyboards list
    let keyboards = match read_keyboards_manifest() {
        Ok(keyboards) => {
            println!("📋 Found {} keyboards to test", keyboards.len());
            keyboards
        }
        Err(e) => {
            panic!("❌ Failed to read keyboards manifest: {}", e);
        }
    };

    if keyboards.is_empty() {
        panic!("❌ No keyboards found in manifest. Please run: ./scripts/collect-qmk-keymaps.sh");
    }

    // Test each keyboard
    let mut results = Vec::new();
    let mut successful_count = 0;
    let mut failed_count = 0;

    for (i, keyboard) in keyboards.iter().enumerate() {
        print!("Testing {}/{}: {}... ", i + 1, keyboards.len(), keyboard);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let result = test_keymap_parsing(keyboard);
        
        if result.success {
            successful_count += 1;
            println!("✅ ({}ms)", result.parse_time_ms);
        } else {
            failed_count += 1;
            println!("❌ ({}ms)", result.parse_time_ms);
            if let Some(error) = &result.error_message {
                println!("   Error: {}", error);
            }
        }
        
        results.push(result);
    }

    // Generate and save report
    let report = generate_compatibility_report(&results);
    fs::write("tests/compatibility_report.md", &report).expect("Failed to write compatibility report");

    // Print summary
    println!("\n🎉 Compatibility testing completed!");
    println!("📊 Results:");
    println!("   • Total: {}", keyboards.len());
    println!("   • Successful: {}", successful_count);
    println!("   • Failed: {}", failed_count);
    println!("   • Success rate: {:.1}%", (successful_count as f64 / keyboards.len() as f64) * 100.0);
    println!("📄 Report saved to: tests/compatibility_report.md");

    // Assert that we have at least some successful tests
    assert!(successful_count > 0, "No keyboards were successfully parsed!");
    
    // Optional: Assert minimum success rate (adjust as needed)
    let success_rate = (successful_count as f64 / keyboards.len() as f64) * 100.0;
    if success_rate < 50.0 {
        println!("⚠️  Warning: Success rate is below 50% ({:.1}%)", success_rate);
    }
}

#[test]
fn test_sample_keyboards() {
    // Test a few known keyboards to ensure basic functionality
    let sample_keyboards = vec![
        "planck_keymap.c",
        "test_keymap.c",
    ];

    for keyboard in sample_keyboards {
        let file_path = format!("tests/files/{}", keyboard);
        if Path::new(&file_path).exists() {
            println!("Testing sample keyboard: {}", keyboard);
            let result = test_keymap_parsing(keyboard);
            assert!(result.success, "Sample keyboard {} failed: {:?}", keyboard, result.error_message);
        }
    }
}
