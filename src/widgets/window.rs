use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Orientation, Button};
use gio::ApplicationFlags;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::info;

use super::notebook::TinkerNotebook;
use super::content::ContentType;

pub struct MainWindow {
    window: ApplicationWindow,
    content_box: GtkBox,
    notebook: Rc<RefCell<TinkerNotebook>>,
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        // Create the main window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Tinker")
            .default_width(1024)
            .default_height(768)
            .build();

        // Create the main vertical box
        let content_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
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

        // Create buttons for different content types
        let new_web_tab = Button::with_label("New Web Tab");
        let new_text_tab = Button::with_label("New Text Tab");
        let new_code_tab = Button::with_label("New Code Tab");
        
        toolbar.append(&new_web_tab);
        toolbar.append(&new_text_tab);
        toolbar.append(&new_code_tab);

        // Create notebook
        let notebook = Rc::new(RefCell::new(TinkerNotebook::new()));
        
        // Add widgets to the content box
        content_box.append(&toolbar);
        content_box.append(notebook.borrow().widget());

        // Add the content box to the window
        window.set_child(Some(&content_box));

        // Connect button signals
        let notebook_clone = notebook.clone();
        new_web_tab.connect_clicked(move |_| {
            let id = notebook_clone.borrow_mut().add_tab_with_type("Web", ContentType::Web);
            if let Some(tab) = notebook_clone.borrow().get_tab(id) {
                info!("Loading example web content");
                let _ = tab.load_content("https://example.com");
            }
        });

        let notebook_clone = notebook.clone();
        new_text_tab.connect_clicked(move |_| {
            let id = notebook_clone.borrow_mut().add_tab_with_type("Text", ContentType::Text);
            if let Some(tab) = notebook_clone.borrow().get_tab(id) {
                info!("Loading example text content");
                let _ = tab.content().set_text_content("Hello, this is a text tab!");
            }
        });

        let notebook_clone = notebook.clone();
        new_code_tab.connect_clicked(move |_| {
            let id = notebook_clone.borrow_mut().add_tab_with_type("Code", ContentType::Code);
            if let Some(tab) = notebook_clone.borrow().get_tab(id) {
                info!("Loading example code content");
                let _ = tab.content().set_text_content(r#"
fn main() {
    println!("Hello, Tinker!");
}
"#);
            }
        });

        // Create initial welcome tab
        let id = notebook.borrow_mut().add_tab_with_type("Welcome", ContentType::Text);
        if let Some(tab) = notebook.borrow().get_tab(id) {
            let _ = tab.content().set_text_content("Welcome to Tinker!\n\nTry creating different types of tabs using the buttons above.");
        }

        Self {
            window,
            content_box,
            notebook,
        }
    }

    pub fn window(&self) -> &ApplicationWindow {
        &self.window
    }

    pub fn content_box(&self) -> &GtkBox {
        &self.content_box
    }

    pub fn notebook(&self) -> Rc<RefCell<TinkerNotebook>> {
        self.notebook.clone()
    }

    pub fn show(&self) {
        self.window.show();
    }
}

pub fn create_application() -> Application {
    Application::builder()
        .application_id("com.github.tinker")
        .flags(ApplicationFlags::empty())
        .build()
        .expect("Failed to initialize GTK application")
} 