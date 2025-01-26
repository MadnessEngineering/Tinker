use gtk::prelude::*;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod widgets;
use widgets::{MainWindow, create_application};

fn main() {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .init();

    info!("Starting Tinker application");

    // Create the GTK application
    let app = create_application();

    // Connect to the activate signal
    app.connect_activate(build_ui);

    // Run the application
    info!("Running GTK application");
    app.run();
}

fn build_ui(app: &gtk::Application) {
    info!("Building UI");
    
    // Create the main window
    let window = MainWindow::new(app);
    
    // Show the window
    window.show();
    
    info!("UI built successfully");
}
