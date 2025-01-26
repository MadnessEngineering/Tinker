use tinker::{MainWindow, create_application};
use gtk::prelude::*;

fn main() {
    let app = create_application();
    
    app.connect_activate(|app| {
        let window = MainWindow::new(app);
        window.show();
    });

    app.run();
}
