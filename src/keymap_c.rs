use crate::config::KeymapConfig;

pub fn parse_keymap_c(source: &str) -> anyhow::Result<KeymapConfig> {
    let source = strip_c_comments(source);

    // Try multiple parsing strategies for better compatibility
    let mut layers: Vec<Vec<String>> = Vec::new();

    // Strategy 1: Look for LAYOUT... ( ... ) blocks
    layers.extend(extract_layout_blocks(&source));

    // Strategy 2: If no layouts found, look for keymap arrays
    if layers.is_empty() {
        layers.extend(extract_keymap_arrays(&source));
    }

    // Strategy 3: Look for const uint16_t PROGMEM keymaps[][]
    if layers.is_empty() {
        layers.extend(extract_progmem_keymaps(&source));
    }

    if layers.is_empty() {
        anyhow::bail!("no LAYOUT(...) blocks found in keymap.c");
    }

    // Try to extract layer bracket names like [NAV], [SYM_SFT]
    let mut names: Vec<String> = Vec::new();
    let mut idx = 0usize;
    for line in source.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            if let Some(end) = line.find(']') {
                let name = line[1..end].to_string();
                names.push(name);
                idx += 1;
                if idx >= layers.len() {
                    break;
                }
            }
        }
    }
    if names.len() < layers.len() {
        while names.len() < layers.len() {
            names.push(format!("Layer {}", names.len()));
        }
    }
    let layer_names = Some(names);
    Ok(KeymapConfig {
        keyboard: "planck".to_string(),
        keymap: "keymap.c".to_string(),
        layers,
        layout: Some("LAYOUT_ortho_4x12".to_string()),
        layer_names,
    })
}

fn strip_c_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            // Line comment - skip until newline
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            // Block comment - skip until */
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i += 2.min(bytes.len().saturating_sub(i));
        } else if bytes[i] == b'"' {
            // String literal - preserve content and handle escaped quotes
            out.push(bytes[i] as char);
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'"' && (i == 0 || bytes[i - 1] != b'\\') {
                    out.push(bytes[i] as char);
                    i += 1;
                    break;
                }
                out.push(bytes[i] as char);
                i += 1;
            }
        } else if bytes[i] == b'\'' {
            // Character literal - preserve content
            out.push(bytes[i] as char);
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\'' && (i == 0 || bytes[i - 1] != b'\\') {
                    out.push(bytes[i] as char);
                    i += 1;
                    break;
                }
                out.push(bytes[i] as char);
                i += 1;
            }
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }

    out
}

fn split_items(inner: &str) -> Vec<String> {
    // Split by commas not inside parentheses (handles MT(...), MO(...), LT(...))
    let mut items = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let mut in_string = false;
    let mut in_char = false;
    let mut escape_next = false;

    for (idx, ch) in inner.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                escape_next = true;
            }
            '"' if !in_char => {
                in_string = !in_string;
            }
            '\'' if !in_string => {
                in_char = !in_char;
            }
            '(' if !in_string && !in_char => {
                depth += 1;
            }
            ')' if !in_string && !in_char => {
                depth -= 1;
            }
            ',' if depth == 0 && !in_string && !in_char => {
                let item = inner[start..idx].trim().to_string();
                if !item.is_empty() {
                    items.push(item);
                }
                start = idx + 1;
            }
            _ => {}
        }
    }

    if start < inner.len() {
        let item = inner[start..].trim().to_string();
        if !item.is_empty() {
            items.push(item);
        }
    }

    items
}

fn extract_layout_blocks(source: &str) -> Vec<Vec<String>> {
    let mut layers: Vec<Vec<String>> = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;

    while i + 6 < bytes.len() {
        if &bytes[i..i + 6] == b"LAYOUT" {
            // Move to first '(' after LAYOUT...
            let mut j = i + 6;
            while j < bytes.len() && bytes[j] != b'(' {
                j += 1;
            }
            if j >= bytes.len() {
                break;
            }

            // Balanced paren capture
            let mut depth = 0usize;
            let start = j + 1;
            let mut end = start;
            while end < bytes.len() {
                match bytes[end] {
                    b'(' => depth += 1,
                    b')' => {
                        if depth == 0 {
                            break;
                        }
                        depth -= 1;
                    }
                    _ => {}
                }
                end += 1;
            }
            if end >= bytes.len() {
                break;
            }

            let inner = &source[start..end];
            let items = split_items(inner)
                .into_iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();

            if !items.is_empty() {
                layers.push(items);
            }
            i = end + 1;
            continue;
        }
        i += 1;
    }

    layers
}

fn extract_keymap_arrays(source: &str) -> Vec<Vec<String>> {
    let mut layers: Vec<Vec<String>> = Vec::new();

    // Look for patterns like: [0] = LAYOUT(...)
    for line in source.lines() {
        let line = line.trim();
        if line.contains("[") && line.contains("]") && line.contains("LAYOUT") {
            if let Some(start) = line.find("LAYOUT") {
                let layout_part = &line[start..];
                if let Some(paren_start) = layout_part.find('(') {
                    let mut depth = 0;
                    let mut end = paren_start;
                    let chars: Vec<char> = layout_part.chars().collect();

                    for (i, &ch) in chars.iter().enumerate().skip(paren_start) {
                        match ch {
                            '(' => depth += 1,
                            ')' => {
                                if depth == 0 {
                                    end = i;
                                    break;
                                }
                                depth -= 1;
                            }
                            _ => {}
                        }
                    }

                    if end > paren_start {
                        let inner = &layout_part[paren_start + 1..end];
                        let items = split_items(inner)
                            .into_iter()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>();

                        if !items.is_empty() {
                            layers.push(items);
                        }
                    }
                }
            }
        }
    }

    layers
}

fn extract_progmem_keymaps(source: &str) -> Vec<Vec<String>> {
    let mut layers: Vec<Vec<String>> = Vec::new();

    // Look for const uint16_t PROGMEM keymaps[][] patterns
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.contains("PROGMEM") && line.contains("keymaps") {
            // Found keymaps declaration, look for the array content
            i += 1;
            while i < lines.len() {
                let line = lines[i].trim();
                if line.starts_with('{') || line.contains("LAYOUT") {
                    // Extract layout from this line
                    if let Some(layout_start) = line.find("LAYOUT") {
                        let layout_part = &line[layout_start..];
                        if let Some(paren_start) = layout_part.find('(') {
                            let mut depth = 0;
                            let mut end = paren_start;
                            let chars: Vec<char> = layout_part.chars().collect();

                            for (j, &ch) in chars.iter().enumerate().skip(paren_start) {
                                match ch {
                                    '(' => depth += 1,
                                    ')' => {
                                        if depth == 0 {
                                            end = j;
                                            break;
                                        }
                                        depth -= 1;
                                    }
                                    _ => {}
                                }
                            }

                            if end > paren_start {
                                let inner = &layout_part[paren_start + 1..end];
                                let items = split_items(inner)
                                    .into_iter()
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect::<Vec<_>>();

                                if !items.is_empty() {
                                    layers.push(items);
                                }
                            }
                        }
                    }
                } else if line.contains('}') && !line.contains("LAYOUT") {
                    // End of keymaps array
                    break;
                }
                i += 1;
            }
        }
        i += 1;
    }

    layers
}

fn _normalize_token(tok: &str) -> String {
    let t = tok.trim().trim_end_matches(',').replace(['\n', '\r'], "");
    if t.is_empty() {
        return t;
    }
    // Common wrappers
    if let Some(inner) = strip_func(&t, "MT")
        .or_else(|| strip_func(&t, "LT"))
        .or_else(|| strip_func(&t, "KC_MT"))
    {
        // Prefer the keycode part (last arg)
        let parts = inner
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();
        if let Some(last) = parts.last() {
            return last.trim_end_matches(',').to_string();
        }
        return inner;
    }
    if let Some(inner) = strip_func(&t, "MO").or_else(|| strip_func(&t, "OSL")) {
        return inner.trim().to_string();
    }
    t
}

#[allow(dead_code)]
fn strip_func(s: &str, name: &str) -> Option<String> {
    let prefix = format!("{}(", name);
    if s.starts_with(&prefix) && s.ends_with(')') {
        return Some(s[prefix.len()..s.len() - 1].to_string());
    }
    None
}
