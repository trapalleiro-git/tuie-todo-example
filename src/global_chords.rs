//! App-wide key chord handler.

use chord_macro::chord;
use tuie::prelude::*;

/// Root widget wrapper that intercepts app-wide key chords.
pub struct GlobalChords {
    inner: Box<dyn Widget>,
}

impl DelegateWidget for GlobalChords {
    tuie::delegate_widget!(inner);

    fn override_on_input(&mut self, queue: &mut InputQueue) -> InputResult {
        let Some(event) = queue.peek() else {
            return InputResult::Rejected;
        };
        match &event.chord {
            chord!(Ctrl + z) => {
                queue.next();
                let _ = tuie::suspend();
            }
            chord!(Ctrl + (c | q)) => {
                queue.next();
                tuie::quit(0);
            }
            _ => return InputResult::Rejected,
        }
        InputResult::Handled
    }
}

impl GlobalChords {
    /// Wraps `inner` in a [`GlobalChords`] handler.
    pub fn new(inner: Box<dyn Widget>) -> Box<Self> {
        Box::new(Self { inner })
    }
}
