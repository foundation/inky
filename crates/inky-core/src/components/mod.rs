mod block_grid;
mod button;
mod callout;
mod center;
mod column;
mod container;
mod divider;
mod helpers;
mod inky;
mod menu;
mod outlook;
mod row;
mod spacer;
mod wrapper;

use scraper::ElementRef;

use crate::config::Config;

// Re-export items used by lib.rs
pub use column::{is_column_element, transform_column_with_position};

/// Transform a single component element into email-safe HTML.
pub fn transform_component(element: &ElementRef, config: &Config) -> Option<String> {
    let tag = element.value().name();
    let comps = &config.components;

    if tag == comps.h_line {
        Some(divider::make_h_line(element))
    } else if tag == comps.columns || tag == "columns" {
        Some(column::make_column(element, config))
    } else if tag == comps.row {
        Some(row::make_row(element))
    } else if tag == comps.button {
        Some(button::make_button(element))
    } else if tag == comps.container {
        Some(container::make_container(element))
    } else if tag == comps.inky {
        Some(inky::make_inky())
    } else if tag == comps.block_grid {
        Some(block_grid::make_block_grid(element))
    } else if tag == comps.menu {
        Some(menu::make_menu(element))
    } else if tag == comps.menu_item {
        Some(menu::make_menu_item(element))
    } else if tag == comps.center {
        Some(center::make_center(element))
    } else if tag == comps.callout {
        Some(callout::make_callout(element))
    } else if tag == comps.spacer {
        Some(spacer::make_spacer(element))
    } else if tag == comps.wrapper {
        Some(wrapper::make_wrapper(element))
    } else if tag == comps.divider {
        Some(divider::make_divider(element))
    } else if tag == comps.outlook {
        Some(outlook::make_outlook(element))
    } else if tag == comps.not_outlook {
        Some(outlook::make_not_outlook(element))
    } else {
        None
    }
}
