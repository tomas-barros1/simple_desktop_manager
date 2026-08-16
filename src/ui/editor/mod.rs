mod footer;
mod section_advanced;
mod section_appearance;
mod section_exec;
mod section_general;

use crate::models::DesktopEntry;
use crate::ui::components::build_separator;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, ScrolledWindow, Window};
use std::cell::RefCell;
use std::rc::Rc;

use footer::build_editor_footer;
use section_advanced::build_advanced_section;
use section_appearance::build_appearance_section;
use section_exec::build_exec_section;
use section_general::build_general_section;

/// Editor panel: cleanly divided into styled sections and a bottom action bar.
pub struct Editor {
    pub root: GtkBox,
    pub state: Rc<RefCell<DesktopEntry>>,
}

impl Editor {
    pub fn new(
        parent: &impl IsA<Window>,
        entry: DesktopEntry,
        on_save: impl Fn(DesktopEntry) + 'static,
        on_cancel: impl Fn() + 'static,
    ) -> Self {
        let parent: Window = parent.upcast_ref().clone();
        let state = Rc::new(RefCell::new(entry));

        let root = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();

        let viewport = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        let form = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .margin_start(24)
            .margin_end(24)
            .margin_top(18)
            .margin_bottom(18)
            .build();

        // Assemble modular sections
        form.append(&build_general_section(&state));
        form.append(&build_exec_section(&parent, &state));
        form.append(&build_appearance_section(&parent, &state));
        form.append(&build_advanced_section(&state));

        viewport.set_child(Some(&form));
        root.append(&viewport);

        // Separator & Footer
        root.append(&build_separator());
        root.append(&build_editor_footer(&state, on_save, on_cancel));

        Self { root, state }
    }

    #[allow(dead_code)]
    pub fn current(&self) -> DesktopEntry {
        self.state.borrow().clone()
    }
}
