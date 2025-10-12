use crate::keyboard::KeyboardLayout;

/// Planck keyboard specific configuration and defaults
pub struct PlanckLayout;

impl PlanckLayout {
    /// Default Planck keyboard dimensions (4 rows, 12 columns)
    pub const ROWS: usize = 4;
    pub const COLS: usize = 12;
    
    /// Default Planck layer names
    pub const DEFAULT_LAYER_NAMES: &'static [&'static str] = &[
        "Base",
        "Lower", 
        "Raise",
        "Adjust",
    ];

    /// Create a default Planck keyboard layout
    pub fn default() -> KeyboardLayout {
        KeyboardLayout::new(
            Self::ROWS,
            Self::COLS,
            Self::DEFAULT_LAYER_NAMES.iter().map(|s| s.to_string()).collect(),
        )
    }

    /// Create a Planck layout with custom layer names
    pub fn with_layer_names(layer_names: Vec<String>) -> KeyboardLayout {
        KeyboardLayout::new(Self::ROWS, Self::COLS, layer_names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planck_default() {
        let layout = PlanckLayout::default();
        assert_eq!(layout.rows, 4);
        assert_eq!(layout.cols, 12);
        assert_eq!(layout.layer_names, vec!["Base", "Lower", "Raise", "Adjust"]);
    }

    #[test]
    fn test_planck_with_custom_layers() {
        let custom_layers = vec!["QWERTY".to_string(), "COLEMAK".to_string()];
        let layout = PlanckLayout::with_layer_names(custom_layers.clone());
        assert_eq!(layout.rows, 4);
        assert_eq!(layout.cols, 12);
        assert_eq!(layout.layer_names, custom_layers);
    }
}
