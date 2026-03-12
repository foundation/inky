/// Configuration for the Inky parser.
#[derive(Debug, Clone)]
pub struct Config {
    pub column_count: u32,
    pub components: ComponentNames,
}

/// Customizable tag names for each Inky component.
#[derive(Debug, Clone)]
pub struct ComponentNames {
    pub button: String,
    pub row: String,
    pub columns: String,
    pub container: String,
    pub callout: String,
    pub inky: String,
    pub block_grid: String,
    pub menu: String,
    pub menu_item: String,
    pub center: String,
    pub spacer: String,
    pub wrapper: String,
    pub h_line: String,
    pub divider: String,
    pub image: String,
    pub outlook: String,
    pub not_outlook: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            column_count: 12,
            components: ComponentNames::default(),
        }
    }
}

impl ComponentNames {
    /// v1 (legacy) tag names.
    pub fn v1() -> Self {
        Self {
            button: "button".into(),
            row: "row".into(),
            columns: "columns".into(), // plural in v1
            container: "container".into(),
            callout: "callout".into(),
            inky: "inky".into(),
            block_grid: "block-grid".into(),
            menu: "menu".into(),
            menu_item: "item".into(),
            center: "center".into(),
            spacer: "spacer".into(),
            wrapper: "wrapper".into(),
            h_line: "h-line".into(), // v1 name
            divider: "divider".into(),
            image: "image".into(),
            outlook: "outlook".into(),
            not_outlook: "not-outlook".into(),
        }
    }
}

impl Default for ComponentNames {
    /// Default accepts both v1 and v2 tag names.
    /// The `columns` field is set to "column" (v2 singular).
    /// v1 tags like `<columns>` and `<h-line>` are also recognized
    /// via the `all_tags()` method which includes both names.
    fn default() -> Self {
        Self {
            button: "button".into(),
            row: "row".into(),
            columns: "column".into(), // v2 singular (v1 "columns" handled via all_tags)
            container: "container".into(),
            callout: "callout".into(),
            inky: "inky".into(),
            block_grid: "block-grid".into(),
            menu: "menu".into(),
            menu_item: "item".into(),
            center: "center".into(),
            spacer: "spacer".into(),
            wrapper: "wrapper".into(),
            h_line: "h-line".into(), // v1 name still handled for compat
            divider: "divider".into(),
            image: "image".into(),
            outlook: "outlook".into(),
            not_outlook: "not-outlook".into(),
        }
    }
}

impl ComponentNames {
    /// Returns all component tag names that the parser should match.
    /// Includes both v1 and v2 names so the parser handles both.
    pub fn all_tags(&self) -> Vec<&str> {
        let mut tags = vec![
            &self.button as &str,
            &self.row as &str,
            &self.columns as &str,
            &self.container as &str,
            &self.callout as &str,
            &self.inky as &str,
            &self.block_grid as &str,
            &self.menu as &str,
            &self.menu_item as &str,
            &self.center as &str,
            &self.spacer as &str,
            &self.wrapper as &str,
            &self.h_line as &str,
            &self.divider as &str,
            // Note: image is NOT in all_tags — it's pre-processed before parsing
            // because html5ever converts <image> to <img>
            &self.outlook as &str,
            &self.not_outlook as &str,
        ];
        // Add v1 aliases if not already present
        if self.columns != "columns" && !tags.contains(&"columns") {
            tags.push("columns");
        }
        tags.dedup();
        tags
    }
}
