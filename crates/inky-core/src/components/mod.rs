mod accordion;
mod alert;
mod badge;
mod block_grid;
mod blockquote;
mod button;
mod callout;
mod card;
mod center;
mod column;
mod container;
mod divider;
mod helpers;
mod hero;
mod inky;
mod menu;
mod outlook;
mod preview;
mod row;
mod social;
mod spacer;
mod video;
mod wrapper;

use scraper::ElementRef;

use crate::config::Config;

// Re-export items used by lib.rs
pub use column::{is_column_element, transform_column_with_position};

type ComponentFn = fn(&ElementRef, &Config) -> String;

/// Build a dispatch table mapping tag names to their transform functions.
fn component_table(config: &Config) -> Vec<(&str, ComponentFn)> {
    let c = &config.components;
    vec![
        (c.h_line.as_str(), |el, _| divider::make_h_line(el)),
        (c.columns.as_str(), |el, cfg| column::make_column(el, cfg)),
        ("columns", |el, cfg| column::make_column(el, cfg)),
        (c.row.as_str(), |el, cfg| row::make_row(el, cfg)),
        (c.button.as_str(), |el, cfg| button::make_button(el, cfg)),
        (c.container.as_str(), |el, cfg| {
            container::make_container(el, cfg)
        }),
        (c.inky.as_str(), |_, _| inky::make_inky()),
        (c.block_grid.as_str(), |el, cfg| {
            block_grid::make_block_grid(el, cfg)
        }),
        (c.menu.as_str(), |el, _| menu::make_menu(el)),
        (c.menu_item.as_str(), |el, _| menu::make_menu_item(el)),
        (c.center.as_str(), |el, _| center::make_center(el)),
        (c.callout.as_str(), |el, _| callout::make_callout(el)),
        (c.spacer.as_str(), |el, _| spacer::make_spacer(el)),
        (c.wrapper.as_str(), |el, cfg| wrapper::make_wrapper(el, cfg)),
        (c.divider.as_str(), |el, _| divider::make_divider(el)),
        (c.outlook.as_str(), |el, _| outlook::make_outlook(el)),
        (c.not_outlook.as_str(), |el, _| {
            outlook::make_not_outlook(el)
        }),
        (c.video.as_str(), |el, _| video::make_video(el)),
        (c.preview.as_str(), |el, _| preview::make_preview(el)),
        (c.hero.as_str(), |el, _| hero::make_hero(el)),
        (c.social.as_str(), |el, _| social::make_social(el)),
        (c.social_link.as_str(), |el, _| social::make_social_link(el)),
        (c.accordion.as_str(), |el, _| accordion::make_accordion(el)),
        (c.accordion_item.as_str(), |el, _| {
            accordion::make_accordion_item(el)
        }),
        (c.card.as_str(), |el, _| card::make_card(el)),
        (c.alert.as_str(), |el, _| alert::make_alert(el)),
        (c.badge.as_str(), |el, _| badge::make_badge(el)),
        (c.blockquote.as_str(), |el, _| {
            blockquote::make_blockquote(el)
        }),
    ]
}

/// Transform a single component element into email-safe HTML.
pub fn transform_component(element: &ElementRef, config: &Config) -> Option<String> {
    let tag = element.value().name();
    for (name, handler) in component_table(config) {
        if tag == name {
            return Some(handler(element, config));
        }
    }
    None
}
