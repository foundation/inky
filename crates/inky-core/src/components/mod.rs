mod accordion;
mod alert;
mod badge;
mod block_grid;
mod blockquote;
mod button;
mod callout;
mod card;
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

// Re-export items used by lib.rs / bindings
pub use column::transform_column_with_position;

/// A component element paired with its already-rendered inner HTML.
///
/// In the single-pass renderer, children are transformed before their
/// parent component runs, so `inner` holds finished output — components
/// must use it instead of `element.inner_html()`.
pub struct El<'a> {
    pub element: ElementRef<'a>,
    pub inner: String,
    /// Classes injected by the renderer (e.g. `float-center` inside `<center>`).
    pub extra_classes: Vec<String>,
}

impl<'a> El<'a> {
    pub fn new(element: ElementRef<'a>, inner: String) -> Self {
        Self {
            element,
            inner,
            extra_classes: Vec::new(),
        }
    }

    pub fn attr(&self, name: &str) -> Option<String> {
        crate::attrs::get_attr(&self.element, name)
    }

    pub fn has_attr(&self, name: &str) -> bool {
        self.element.value().attr(name).is_some()
    }

    pub fn classes(&self) -> Vec<String> {
        let mut classes = crate::attrs::get_classes(&self.element);
        classes.extend(self.extra_classes.iter().cloned());
        classes
    }

    pub fn has_class(&self, class: &str) -> bool {
        self.classes().iter().any(|c| c == class)
    }

    /// Passthrough attributes as ` key="value"` pairs (ignored attrs filtered).
    pub fn attrs(&self) -> String {
        crate::attrs::get_attrs(&self.element)
    }

    pub fn inner(&self) -> &str {
        &self.inner
    }
}

/// Context threaded through the render walk to component functions.
pub struct RenderCtx<'a> {
    pub config: &'a Config,
    /// True anywhere inside a `<center>` component (menu items add float-center).
    pub inside_center: bool,
}

type ComponentFn = fn(&El, &RenderCtx) -> String;

/// Build a dispatch table mapping tag names to their transform functions.
fn component_table(config: &Config) -> Vec<(&str, ComponentFn)> {
    let c = &config.components;
    vec![
        (c.h_line.as_str(), |el, _| divider::make_h_line(el)),
        (c.columns.as_str(), |el, ctx| {
            column::make_column(el, ctx.config)
        }),
        ("columns", |el, ctx| column::make_column(el, ctx.config)),
        (c.row.as_str(), |el, ctx| row::make_row(el, ctx.config)),
        (c.button.as_str(), |el, ctx| {
            button::make_button(el, ctx.config)
        }),
        (c.container.as_str(), |el, ctx| {
            container::make_container(el, ctx.config)
        }),
        (c.inky.as_str(), |_, _| inky::make_inky()),
        (c.block_grid.as_str(), |el, ctx| {
            block_grid::make_block_grid(el, ctx.config)
        }),
        (c.menu.as_str(), |el, _| menu::make_menu(el)),
        (c.menu_item.as_str(), |el, ctx| {
            menu::make_menu_item(el, ctx)
        }),
        (c.callout.as_str(), |el, _| callout::make_callout(el)),
        (c.spacer.as_str(), |el, _| spacer::make_spacer(el)),
        (c.wrapper.as_str(), |el, ctx| {
            wrapper::make_wrapper(el, ctx.config)
        }),
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
pub fn transform_component(el: &El, ctx: &RenderCtx) -> Option<String> {
    let tag = el.element.value().name();
    for (name, handler) in component_table(ctx.config) {
        if tag == name {
            return Some(handler(el, ctx));
        }
    }
    None
}
