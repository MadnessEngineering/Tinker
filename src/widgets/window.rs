use gtk::prelude::*;
use gtk::{self, Application, ApplicationWindow};
use gio::ApplicationFlags;

pub struct MainWindow {
    window: ApplicationWindow,
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Tinker")
            .default_width(800)
            .default_height(600)
            .build();

        Self { window }
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