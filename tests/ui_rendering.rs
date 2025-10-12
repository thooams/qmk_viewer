use qmk_viewer::config::KeymapConfig;
use qmk_viewer::keycodes;
use qmk_viewer::keymap_c;
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct UIRenderingResult {
    keyboard_name: String,
    layout_creation_success: bool,
    #[allow(dead_code)]
    dimensions_detected: bool,
    rows: Option<usize>,
    cols: Option<usize>,
    total_keys: Option<usize>,
    error_message: Option<String>,
    render_time_ms: u64,
}

impl UIRenderingResult {
    fn success(
        keyboard_name: String,
        rows: usize,
        cols: usize,
        total_keys: usize,
        render_time_ms: u64,
    ) -> Self {
        Self {
            keyboard_name,
            layout_creation_success: true,
            dimensions_detected: true,
            rows: Some(rows),
            cols: Some(cols),
            total_keys: Some(total_keys),
            error_message: None,
            render_time_ms,
        }
    }

    fn failure(keyboard_name: String, error_message: String, render_time_ms: u64) -> Self {
        Self {
            keyboard_name,
            layout_creation_success: false,
            dimensions_detected: false,
            rows: None,
            cols: None,
            total_keys: None,
            error_message: Some(error_message),
            render_time_ms,
        }
    }
}

fn test_ui_rendering(keyboard_name: &str) -> UIRenderingResult {
    let start_time = std::time::Instant::now();
    let file_path = format!("tests/files/{}", keyboard_name);

    match fs::read_to_string(&file_path) {
        Ok(content) => {
            match keymap_c::parse_keymap_c(&content) {
                Ok(keymap) => {
                    // Extract clean keyboard name
                    let clean_name = keyboard_name.replace("_keymap.c", "");

                    // Try to create keyboard layout
                    match create_keyboard_layout(&keymap) {
                        Ok((rows, cols, total_keys)) => {
                            let render_time = start_time.elapsed().as_millis() as u64;
                            UIRenderingResult::success(
                                clean_name,
                                rows,
                                cols,
                                total_keys,
                                render_time,
                            )
                        }
                        Err(e) => {
                            let render_time = start_time.elapsed().as_millis() as u64;
                            let clean_name = keyboard_name.replace("_keymap.c", "");
                            UIRenderingResult::failure(
                                clean_name,
                                format!("Layout creation error: {}", e),
                                render_time,
                            )
                        }
                    }
                }
                Err(e) => {
                    let render_time = start_time.elapsed().as_millis() as u64;
                    let clean_name = keyboard_name.replace("_keymap.c", "");
                    UIRenderingResult::failure(
                        clean_name,
                        format!("Parse error: {}", e),
                        render_time,
                    )
                }
            }
        }
        Err(e) => {
            let render_time = start_time.elapsed().as_millis() as u64;
            let clean_name = keyboard_name.replace("_keymap.c", "");
            UIRenderingResult::failure(clean_name, format!("File read error: {}", e), render_time)
        }
    }
}

fn create_keyboard_layout(keymap: &KeymapConfig) -> Result<(usize, usize, usize), String> {
    if keymap.layers.is_empty() {
        return Err("No layers found".to_string());
    }

    let first_layer = &keymap.layers[0];
    if first_layer.is_empty() {
        return Err("First layer is empty".to_string());
    }

    // Try to detect dimensions from the keymap
    let (rows, cols) = detect_keyboard_dimensions(first_layer)?;
    let total_keys = rows * cols;

    // Validate that we have enough keys
    if first_layer.len() < total_keys {
        return Err(format!(
            "Insufficient keys: expected {}, found {}",
            total_keys,
            first_layer.len()
        ));
    }

    // Test keycode translation for a sample of keys
    test_keycode_translation(first_layer)?;

    Ok((rows, cols, total_keys))
}

fn detect_keyboard_dimensions(keys: &[String]) -> Result<(usize, usize), String> {
    // Common keyboard layouts and their dimensions
    let common_layouts = vec![
        (1, 1),  // 1 key
        (1, 2),  // 2 keys
        (1, 3),  // 3 keys
        (1, 4),  // 4 keys
        (1, 5),  // 5 keys
        (2, 2),  // 2x2
        (2, 3),  // 2x3
        (2, 4),  // 2x4
        (2, 5),  // 2x5
        (3, 3),  // 3x3
        (3, 4),  // 3x4
        (3, 5),  // 3x5
        (4, 4),  // 4x4
        (4, 5),  // 4x5
        (4, 6),  // 4x6
        (4, 8),  // 4x8
        (4, 10), // 4x10
        (4, 12), // 4x12
        (4, 13), // 4x13
        (4, 14), // 4x14
        (4, 15), // 4x15
        (5, 12), // 5x12
        (5, 13), // 5x13
        (5, 14), // 5x14
        (5, 15), // 5x15
        (6, 15), // 6x15
        (6, 16), // 6x16
        (6, 17), // 6x17
        (6, 18), // 6x18
        (6, 19), // 6x19
        (6, 20), // 6x20
    ];

    let key_count = keys.len();

    // Find the best matching layout
    for (rows, cols) in common_layouts {
        if rows * cols == key_count {
            return Ok((rows, cols));
        }
    }

    // If no exact match, try to find a reasonable approximation
    // Look for factors of the key count
    for rows in 1..=key_count {
        if key_count.is_multiple_of(rows) {
            let cols = key_count / rows;
            if rows <= 10 && cols <= 25 {
                // Reasonable keyboard dimensions
                return Ok((rows, cols));
            }
        }
    }

    Err(format!(
        "Could not determine keyboard dimensions for {} keys",
        key_count
    ))
}

fn test_keycode_translation(keys: &[String]) -> Result<(), String> {
    // Test translation of a sample of keys to ensure no panics
    let sample_size = std::cmp::min(10, keys.len());

    for key in keys.iter().take(sample_size) {
        // This should not panic
        let _translated = keycodes::translate_token(key);
    }

    // Test some common keycodes that might cause issues
    let test_keycodes = vec![
        "KC_A",
        "KC_1",
        "KC_ENT",
        "KC_SPC",
        "KC_LSFT",
        "KC_LCTL",
        "MO(1)",
        "LT(1, KC_A)",
        "MT(MOD_LSFT, KC_A)",
        "TG(1)",
        "RGB_TOG",
        "RGB_MODE_FORWARD",
        "QK_BOOT",
        "EE_CLR",
    ];

    for keycode in test_keycodes {
        let _translated = keycodes::translate_token(keycode);
    }

    Ok(())
}

fn generate_ui_report(results: &[UIRenderingResult]) -> String {
    let total_tests = results.len();
    let successful_tests = results.iter().filter(|r| r.layout_creation_success).count();
    let failed_tests = total_tests - successful_tests;
    let success_rate = if total_tests > 0 {
        (successful_tests as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    let mut report = String::new();
    report.push_str("# QMK UI Rendering Compatibility Report\n\n");
    report.push_str(&format!(
        "Generated on: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Summary
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- **Total keyboards tested**: {}\n", total_tests));
    report.push_str(&format!("- **Successful**: {}\n", successful_tests));
    report.push_str(&format!("- **Failed**: {}\n", failed_tests));
    report.push_str(&format!("- **Success rate**: {:.1}%\n\n", success_rate));

    // Performance stats
    let total_render_time: u64 = results.iter().map(|r| r.render_time_ms).sum();
    let avg_render_time = if total_tests > 0 {
        total_render_time as f64 / total_tests as f64
    } else {
        0.0
    };

    report.push_str("## Performance\n\n");
    report.push_str(&format!(
        "- **Total render time**: {}ms\n",
        total_render_time
    ));
    report.push_str(&format!(
        "- **Average render time**: {:.1}ms\n\n",
        avg_render_time
    ));

    // Successful keyboards
    let successful: Vec<_> = results
        .iter()
        .filter(|r| r.layout_creation_success)
        .collect();
    if !successful.is_empty() {
        report.push_str("## Successful UI Rendering\n\n");
        report.push_str("| Keyboard | Rows | Cols | Total Keys | Render Time (ms) |\n");
        report.push_str("|----------|------|------|------------|------------------|\n");

        for result in &successful {
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

    // Failed keyboards
    let failed: Vec<_> = results
        .iter()
        .filter(|r| !r.layout_creation_success)
        .collect();
    if !failed.is_empty() {
        report.push_str("## Failed UI Rendering\n\n");
        report.push_str("| Keyboard | Error | Render Time (ms) |\n");
        report.push_str("|----------|-------|------------------|\n");

        for result in &failed {
            let error = result.error_message.as_deref().unwrap_or("Unknown error");
            report.push_str(&format!(
                "| {} | {} | {} |\n",
                result.keyboard_name, error, result.render_time_ms
            ));
        }
        report.push('\n');
    }

    report
}

#[test]
fn test_ui_rendering_sample() {
    println!("🎨 Starting UI rendering compatibility tests...");

    // Check if manifest exists
    if !Path::new("tests/files/keyboards.txt").exists() {
        panic!("❌ Keyboards manifest not found. Please run: ./scripts/collect-qmk-keymaps.sh");
    }

    // Read a sample of keyboards for UI testing (limit to 20 for performance)
    let keyboards = match read_sample_keyboards() {
        Ok(keyboards) => {
            println!(
                "📋 Testing UI rendering for {} sample keyboards",
                keyboards.len()
            );
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
        print!("Testing UI {}/{}: {}... ", i + 1, keyboards.len(), keyboard);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let result = test_ui_rendering(keyboard);

        if result.layout_creation_success {
            successful_count += 1;
            println!("✅ ({}ms)", result.render_time_ms);
        } else {
            failed_count += 1;
            println!("❌ ({}ms)", result.render_time_ms);
            if let Some(error) = &result.error_message {
                println!("   Error: {}", error);
            }
        }

        results.push(result);
    }

    // Generate and save report
    let report = generate_ui_report(&results);
    fs::write("tests/ui_rendering_report.md", &report)
        .expect("Failed to write UI rendering report");

    // Print summary
    println!("\n🎉 UI rendering testing completed!");
    println!("📊 Results:");
    println!("   • Total: {}", keyboards.len());
    println!("   • Successful: {}", successful_count);
    println!("   • Failed: {}", failed_count);
    println!(
        "   • Success rate: {:.1}%",
        (successful_count as f64 / keyboards.len() as f64) * 100.0
    );
    println!("📄 Report saved to: tests/ui_rendering_report.md");

    // Assert that we have at least some successful tests
    assert!(
        successful_count > 0,
        "No keyboards were successfully rendered!"
    );
}

fn read_sample_keyboards() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let manifest_path = "tests/files/keyboards.txt";
    let content = fs::read_to_string(manifest_path)?;

    let all_keyboards: Vec<String> = content
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect();

    // Take a sample of keyboards for UI testing (first 20)
    let sample_size = std::cmp::min(20, all_keyboards.len());
    Ok(all_keyboards.into_iter().take(sample_size).collect())
}

#[test]
fn test_known_keyboards_ui() {
    // Test specific known keyboards that should work
    let known_keyboards = vec!["planck_keymap.c", "test_keymap.c"];

    for keyboard in known_keyboards {
        let file_path = format!("tests/files/{}", keyboard);
        if Path::new(&file_path).exists() {
            println!("Testing known keyboard UI: {}", keyboard);
            let result = test_ui_rendering(keyboard);
            assert!(
                result.layout_creation_success,
                "Known keyboard {} failed UI rendering: {:?}",
                keyboard, result.error_message
            );
        }
    }
}
