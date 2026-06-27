//! Button List Item.
//!
use chord_macro::chord;
use std::cell::Cell;
use tuie::prelude::*;

thread_local! {
    static ACCENT: Cell<Style> = Cell::new(Style::new().fg(Color::BLUE));
}

/// Button Item that emits [`ClickEvent`].
pub struct ButtonItem {
    root: Box<Pane>,
    color: Color,
}

impl DelegateWidget for ButtonItem {
    tuie::delegate_widget!(root);

    fn override_is_focusable(&self) -> bool {
        true
    }

    fn after_on_state_change(&mut self, state: WidgetState) {
        let accent = ACCENT.with(|c| c.get());
        let style = match state {
            WidgetState::Focused | WidgetState::FocusedHover => accent.bold(),
            WidgetState::Active => accent,
            _ => Style::new().fg(self.color),
        };
        self.root.set_style(style);
    }

    fn override_on_input(&mut self, queue: &mut InputQueue) -> InputResult {
        let Some(event) = queue.next() else {
            return InputResult::Rejected;
        };
        match &event.chord {
            chord!(Enter | Space) => {
                tuie::emit(self.get_id(), ClickEvent);
            }

            chord!(LeftClick) => {
                tuie::focus_widget(self.get_id());
            }

            chord!(LeftRelease) => {
                let size = self.get_rect_size();
                if Axis2D::all(|a| event.cell()[a] >= 0 && event.cell()[a] < size[a] as i32) {
                    tuie::emit(self.get_id(), ClickEvent);
                }
            }

            _ => return InputResult::Rejected,
        }
        InputResult::Handled
    }
}

impl ButtonItem {
    /// Creates button item with the given label.
    pub fn new(label: &str, color: Color) -> Box<Self> {
        let root = Pane::new()
            .style(Style::new().fg(color))
            .horizontal()
            .gap(1)
            .children([Text::new().content(label)]);
        Box::new(Self { root, color })
    }
}
