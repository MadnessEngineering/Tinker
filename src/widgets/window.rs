use gtk::prelude::*;
use gtk::{self, Application, ApplicationWindow, Box as GtkBox, Button, Orientation};
use gio::{ApplicationFlags, SimpleAction};

use super::notebook::TinkerNotebook;
use super::web_view::WebContent;

pub struct MainWindow {
    window: ApplicationWindow,
    notebook: TinkerNotebook,
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Tinker")
            .default_width(800)
            .default_height(600)
            .build();

        // Create main vertical box
        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .build();

        // Create toolbar
        let toolbar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .margin_start(6)
            .margin_end(6)
            .margin_top(6)
            .margin_bottom(6)
            .build();

        // Create new tab button
        let new_tab_button = Button::with_label("New Tab");
        toolbar.append(&new_tab_button);

        // Create notebook
        let notebook = TinkerNotebook::new();
        
        // Add default tab with web content
        let web_content = WebContent::new();
        web_content.load_url("https://example.com");
        notebook.add_tab("New Tab", web_content.widget());

        // Add widgets to main box
        main_box.append(&toolbar);
        main_box.append(notebook.widget());

        // Add main box to window
        window.set_child(Some(&main_box));

        // Connect new tab button
        let notebook_clone = notebook.widget().clone();
        new_tab_button.connect_clicked(move |_| {
            let web_content = WebContent::new();
            web_content.load_url("https://example.com");
            let label = gtk::Label::new(Some("New Tab"));
            notebook_clone.append_page(web_content.widget(), Some(&label));
            web_content.widget().show();
        });

        // Add tab actions to the application
        let action = SimpleAction::new("tab.new", None);
        let notebook_clone = notebook.widget().clone();
        action.connect_activate(move |_, _| {
            let web_content = WebContent::new();
            web_content.load_url("https://example.com");
            let label = gtk::Label::new(Some("New Tab"));
            notebook_clone.append_page(web_content.widget(), Some(&label));
            web_content.widget().show();
        });
        app.add_action(&action);

        Self { window, notebook }
    }

    pub fn show(&self) {
        self.window.show();
    }
}

pub fn create_application() -> Application {
    gtk::init().expect("Failed to initialize GTK");
    
    let app = Application::builder()
        .application_id("com.github.tinker")
        .flags(ApplicationFlags::empty())
        .build();
    app
} 