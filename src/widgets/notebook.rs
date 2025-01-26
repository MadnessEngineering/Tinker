use gtk::prelude::*;
use gtk::{self, Notebook, Box as GtkBox, Orientation};
use gio::SimpleAction;

use super::tab_label::TabLabel;
use super::web_view::WebContent;

pub struct TinkerNotebook {
    notebook: Notebook,
}

impl TinkerNotebook {
    pub fn new() -> Self {
        let notebook = Notebook::builder()
            .show_border(true)
            .scrollable(true)
            .build();

        Self { notebook }
    }

    pub fn widget(&self) -> &Notebook {
        &self.notebook
    }

    pub fn add_tab(&self, title: &str, content: &impl IsA<gtk::Widget>) {
        let tab_label = TabLabel::new(title);
        self.notebook.append_page(content, Some(tab_label.widget()));
        content.show();
        self.notebook.set_current_page(Some(self.notebook.n_pages() - 1));

        // Add tab actions
        let notebook_clone = self.notebook.clone();
        let content_clone = content.clone();
        
        // New tab action
        let action = SimpleAction::new("tab.new", None);
        action.connect_activate(move |_, _| {
            let new_content = WebContent::new();
            new_content.load_url("https://example.com");
            let new_label = TabLabel::new("New Tab");
            notebook_clone.append_page(new_content.widget(), Some(new_label.widget()));
            new_content.widget().show();
            notebook_clone.set_current_page(Some(notebook_clone.n_pages() - 1));
        });

        // Close tab action
        let action = SimpleAction::new("tab.close", None);
        let notebook_clone = self.notebook.clone();
        action.connect_activate(move |_, _| {
            if let Some(page_num) = notebook_clone.page_num(&content_clone) {
                notebook_clone.remove_page(Some(page_num));
            }
        });

        // Duplicate tab action
        let action = SimpleAction::new("tab.duplicate", None);
        let notebook_clone = self.notebook.clone();
        let content_clone = content.clone();
        let title = title.to_string();
        action.connect_activate(move |_, _| {
            let new_content = content_clone.clone();
            let new_label = TabLabel::new(&title);
            notebook_clone.append_page(&new_content, Some(new_label.widget()));
            new_content.show();
            notebook_clone.set_current_page(Some(notebook_clone.n_pages() - 1));
        });

        // Reload tab action
        let action = SimpleAction::new("tab.reload", None);
        let content_clone = content.clone();
        action.connect_activate(move |_, _| {
            if let Some(web_content) = content_clone.downcast_ref::<GtkBox>() {
                if let Some(web_view) = web_content.first_child() {
                    if let Some(web_content) = web_view.downcast_ref::<WebContent>() {
                        web_content.reload();
                    }
                }
            }
        });
    }
} 
