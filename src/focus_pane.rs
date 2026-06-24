//! Bordered pane that highlights itself when selected or active.
use std::cell::Cell;

use tuie::render::border;
use tuie::{delegate_field, field, prelude::*};

thread_local! {
    static ACCENT: Cell<Style> = Cell::new(Style::new().fg(Color::BLUE));
}

/// Bordered [`Pane`] wrapper that highlights its border when focused.
pub(crate) struct FocusPane {
    pane: Box<Pane>,
    border_style: Style,
    selected_border_style: Option<Style>,
}

impl FocusPane {
    fn refresh_border(&mut self) {
        let cfg = border::config::get();
        let focused = tuie::in_focus_chain(self.pane.get_id());
        if focused {
            let style = self
                .selected_border_style
                .unwrap_or_else(|| cfg.selected_style.apply(ACCENT.with(|c| c.get())));
            self.pane.set_border_style(style);
        } else {
            self.pane.set_border_style(self.border_style);
        }
    }

    fn sync_border_style(&mut self) {
        self.refresh_border();
    }
}

impl DelegateWidget for FocusPane {
    tuie::delegate_widget!(pane);

    fn after_on_state_change(&mut self, _widget_state: WidgetState) {
        self.pane.dirty_layout();
    }

    fn after_before_layout(&mut self) {
        self.refresh_border();
    }
}

impl FocusPane {
    /// Creates an empty bordered focus pane.
    pub(crate) fn new() -> Box<Self> {
        let mut pane = Pane::new();
        pane.set_bordered(true);
        Box::new(Self {
            pane,
            border_style: Style::new(),
            selected_border_style: None,
        })
    }

    /// Appends a child widget.
    pub(crate) fn add_child(&mut self, widget: Box<dyn Widget>) {
        self.pane.add_child(widget);
    }

    /// Appends `children` to the pane.
    pub(crate) fn children<const N: usize>(mut self: Box<Self>, children: [Box<dyn Widget>; N]) -> Box<Self> {
        for child in children {
            self.pane.add_child(child);
        }
        self
    }

    field!(border_style: Style; sync_border_style);
    field!(selected_border_style: Option<Style>);

    delegate_field!(orientation: Axis2D => pane);
    delegate_field!(gap: u8 => pane);
}
