use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Orientation, Button};
use gio::ApplicationFlags;
use std::cell::RefCell;
use std::rc::Rc;

use super::notebook::TinkerNotebook;

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

        // Create new tab button
        let new_tab_button = Button::with_label("New Tab");
        toolbar.append(&new_tab_button);

        // Create notebook
        let notebook = Rc::new(RefCell::new(TinkerNotebook::new()));
        
        // Add widgets to the content box
        content_box.append(&toolbar);
        content_box.append(notebook.borrow().widget());

        // Add the content box to the window
        window.set_child(Some(&content_box));

        // Connect new tab button
        let notebook_clone = notebook.clone();
        new_tab_button.connect_clicked(move |_| {
            notebook_clone.borrow_mut().add_tab("New Tab");
        });

        // Create initial tab
        notebook.borrow_mut().add_tab("Welcome");

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