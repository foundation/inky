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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            column_count: 12,
            components: ComponentNames::default(),
        }
    }
}

impl Default for ComponentNames {
    fn default() -> Self {
        Self {
            button: "button".into(),
            row: "row".into(),
            columns: "columns".into(),
            container: "container".into(),
            callout: "callout".into(),
            inky: "inky".into(),
            block_grid: "block-grid".into(),
            menu: "menu".into(),
            menu_item: "item".into(),
            center: "center".into(),
            spacer: "spacer".into(),
            wrapper: "wrapper".into(),
            h_line: "h-line".into(),
        }
    }
}

impl ComponentNames {
    /// Returns all component tag names as a slice.
    pub fn all_tags(&self) -> Vec<&str> {
        vec![
            &self.button,
            &self.row,
            &self.columns,
            &self.container,
            &self.callout,
            &self.inky,
            &self.block_grid,
            &self.menu,
            &self.menu_item,
            &self.center,
            &self.spacer,
            &self.wrapper,
            &self.h_line,
        ]
    }
}
