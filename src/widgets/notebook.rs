use gtk::prelude::*;
use gtk::{Notebook as GtkNotebook, Box as GtkBox};
use std::collections::HashMap;
use tracing::debug;

use super::tab::{Tab, TabLabel};

pub struct TinkerNotebook {
    notebook: GtkNotebook,
    tabs: HashMap<u32, Tab>,
    next_id: u32,
}

impl TinkerNotebook {
    pub fn new() -> Self {
        let notebook = GtkNotebook::builder()
            .scrollable(true)
            .show_border(false)
            .build();

        Self {
            notebook,
            tabs: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn widget(&self) -> &GtkNotebook {
        &self.notebook
    }

    pub fn add_tab(&mut self, title: &str) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        // Create new tab and label
        let tab = Tab::new(title);
        let tab_label = TabLabel::new(title);

        // Store tab reference
        let tab_content = tab.widget().clone();
        self.tabs.insert(id, tab);

        // Connect close button
        let notebook_weak = self.notebook.downgrade();
        tab_label.connect_close(move || {
            if let Some(notebook) = notebook_weak.upgrade() {
                if let Some(num) = notebook.page_num(&tab_content) {
                    notebook.remove_page(Some(num));
                }
            }
        });

        // Add the tab to the notebook
        self.notebook.append_page(&tab_content, Some(tab_label.widget()));
        
        debug!("Added new tab with id: {}", id);
        
        // Show all widgets
        tab_content.show();
        
        id
    }

    pub fn remove_tab(&mut self, id: u32) -> bool {
        if let Some(tab) = self.tabs.remove(&id) {
            if let Some(num) = self.notebook.page_num(tab.widget()) {
                self.notebook.remove_page(Some(num));
                debug!("Removed tab with id: {}", id);
                return true;
            }
        }
        false
    }

    pub fn get_tab(&self, id: u32) -> Option<&Tab> {
        self.tabs.get(&id)
    }

    pub fn get_tab_mut(&mut self, id: u32) -> Option<&mut Tab> {
        self.tabs.get_mut(&id)
    }

    pub fn get_current_tab(&self) -> Option<u32> {
        let current_page = self.notebook.current_page()?;
        let current_widget = self.notebook.nth_page(Some(current_page))?;
        
        self.tabs.iter()
            .find(|(_, tab)| tab.widget() == &current_widget)
            .map(|(id, _)| *id)
    }

    pub fn set_tab_title(&mut self, id: u32, title: &str) {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.set_title(title);
            if let Some(num) = self.notebook.page_num(tab.widget()) {
                if let Some(tab_label) = self.notebook.tab_label(tab.widget()) {
                    if let Some(box_label) = tab_label.downcast_ref::<GtkBox>() {
                        if let Some(label) = box_label.first_child() {
                            if let Some(label) = label.downcast_ref::<gtk::Label>() {
                                label.set_text(title);
                            }
                        }
                    }
                }
            }
        }
    }
} 