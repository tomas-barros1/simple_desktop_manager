mod footer;
mod section_advanced;
mod section_appearance;
mod section_exec;
mod section_general;

use crate::models::DesktopEntry;
use crate::ui::components::build_separator;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Window};
use libadwaita::prelude::*;
use libadwaita::{PreferencesPage, ToastOverlay};
use std::cell::RefCell;
use std::rc::Rc;

use footer::build_editor_footer;
use section_advanced::build_advanced_section;
use section_appearance::build_appearance_section;
use section_exec::build_exec_section;
use section_general::build_general_section;

/// Editor panel: cleanly divided into LibAdwaita preferences groups and an action footer.
pub struct Editor {
    pub root: GtkBox,
    pub state: Rc<RefCell<DesktopEntry>>,
}

impl Editor {
    pub fn new(
        parent: &impl IsA<Window>,
        entry: DesktopEntry,
        toast_overlay: &ToastOverlay,
        on_save: impl Fn(DesktopEntry) + 'static,
        on_cancel: impl Fn() + 'static,
    ) -> Self {
        let parent: Window = parent.upcast_ref().clone();
        let state = Rc::new(RefCell::new(entry));

        let root = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();

        let page = PreferencesPage::new();

        // Assemble modular Adwaita preferences groups
        page.add(&build_general_section(&state));
        page.add(&build_exec_section(&parent, &state));
        page.add(&build_appearance_section(&parent, &state));
        page.add(&build_advanced_section(&state));

        root.append(&page);

        // Separator & Footer
        root.append(&build_separator());
        root.append(&build_editor_footer(&state, toast_overlay, on_save, on_cancel));

        Self { root, state }
    }

    #[allow(dead_code)]
    pub fn current(&self) -> DesktopEntry {
        self.state.borrow().clone()
    }
}
