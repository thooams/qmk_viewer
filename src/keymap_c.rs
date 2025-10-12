use crate::config::KeymapConfig;

pub fn parse_keymap_c(source: &str) -> anyhow::Result<KeymapConfig> {
    let source = strip_c_comments(source);
    // Extract all LAYOUT... ( ... ) blocks, preserving commas
    // Simple approach: find "LAYOUT" tokens and then capture balanced parentheses
    let mut layers: Vec<Vec<String>> = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while i + 6 < bytes.len() {
        if &bytes[i..i + 6] == b"LAYOUT" {
            // Move to first '(' after LAYOUT...
            let mut j = i + 6;
            while j < bytes.len() && bytes[j] != b'(' { j += 1; }
            if j >= bytes.len() { break; }
            // Balanced paren capture
            let mut depth = 0usize;
            let start = j + 1;
            let mut end = start;
            while end < bytes.len() {
                match bytes[end] {
                    b'(' => depth += 1,
                    b')' => { if depth == 0 { break; } depth -= 1; },
                    _ => {}
                }
                end += 1;
            }
            if end >= bytes.len() { break; }
            let inner = &source[start..end];
            let items = split_items(inner)
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect::<Vec<_>>();
            if !items.is_empty() {
                layers.push(items);
            }
            i = end + 1;
            continue;
        }
        i += 1;
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
                if idx >= layers.len() { break; }
            }
        }
    }
    if names.len() < layers.len() {
        while names.len() < layers.len() { names.push(format!("Layer {}", names.len())); }
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
            // line comment
            while i < bytes.len() && bytes[i] != b'\n' { i += 1; }
        } else if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            // block comment
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') { i += 1; }
            i += 2.min(bytes.len().saturating_sub(i));
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

fn split_items(inner: &str) -> Vec<String> {
    // Split by commas not inside parentheses (handles MT(...), MO(...))
    let mut items = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (idx, ch) in inner.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                items.push(inner[start..idx].to_string());
                start = idx + 1;
            }
            _ => {}
        }
    }
    if start < inner.len() {
        items.push(inner[start..].to_string());
    }
    items
}

fn normalize_token(tok: &str) -> String {
    let t = tok
        .trim()
        .trim_end_matches(',')
        .replace('\n', "")
        .replace('\r', "");
    if t.is_empty() { return t; }
    // Common wrappers
    if let Some(inner) = strip_func(&t, "MT")
        .or_else(|| strip_func(&t, "LT"))
        .or_else(|| strip_func(&t, "KC_MT"))
    {
        // Prefer the keycode part (last arg)
        let parts = inner.split(',').map(|s| s.trim().to_string()).collect::<Vec<String>>();
        if let Some(last) = parts.last() { return last.trim_end_matches(',').to_string(); }
        return inner;
    }
    if let Some(inner) = strip_func(&t, "MO").or_else(|| strip_func(&t, "OSL")) {
        return inner.trim().to_string();
    }
    t
}

fn strip_func(s: &str, name: &str) -> Option<String> {
    let prefix = format!("{}(", name);
    if s.starts_with(&prefix) && s.ends_with(')') {
        return Some(s[prefix.len()..s.len()-1].to_string());
    }
    None
}


