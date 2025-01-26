use gtk::prelude::*;
use gtk::{self, Box as GtkBox, Entry, Button, Orientation};
use webkit::prelude::*;
use webkit::WebView as WebKitView;
use webkit::WebContext;
use webkit::Settings;

pub struct WebContent {
    container: GtkBox,
    url_bar: Entry,
    web_view: WebKitView,
}

impl WebContent {
    pub fn new() -> Self {
        // Create main container
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .build();

        // Create URL bar container
        let url_container = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .margin_start(6)
            .margin_end(6)
            .margin_top(6)
            .margin_bottom(6)
            .build();

        // Create URL entry
        let url_bar = Entry::builder()
            .hexpand(true)
            .placeholder_text("Enter URL")
            .build();

        // Create navigation buttons
        let back_button = Button::with_label("←");
        let forward_button = Button::with_label("→");
        let refresh_button = Button::with_label("⟳");

        // Create web view
        let context = WebContext::default().unwrap();
        let settings = Settings::new();
        settings.set_enable_developer_extras(true);
        
        let web_view = WebKitView::builder()
            .web_context(&context)
            .settings(&settings)
            .build();

        web_view.set_vexpand(true);

        // Add URL bar widgets
        url_container.append(&back_button);
        url_container.append(&forward_button);
        url_container.append(&refresh_button);
        url_container.append(&url_bar);

        // Add all widgets to main container
        container.append(&url_container);
        container.append(&web_view);

        // Connect signals
        let web_view_clone = web_view.clone();
        url_bar.connect_activate(move |entry| {
            let url = entry.text();
            if !url.starts_with("http://") && !url.starts_with("https://") {
                web_view_clone.load_uri(&format!("https://{}", url));
            } else {
                web_view_clone.load_uri(&url);
            }
        });

        let web_view_clone = web_view.clone();
        back_button.connect_clicked(move |_| {
            if web_view_clone.can_go_back() {
                web_view_clone.go_back();
            }
        });

        let web_view_clone = web_view.clone();
        forward_button.connect_clicked(move |_| {
            if web_view_clone.can_go_forward() {
                web_view_clone.go_forward();
            }
        });

        let web_view_clone = web_view.clone();
        refresh_button.connect_clicked(move |_| {
            web_view_clone.reload();
        });

        let url_bar_clone = url_bar.clone();
        web_view.connect_load_changed(move |web_view, _| {
            if let Some(uri) = web_view.uri() {
                url_bar_clone.set_text(&uri);
            }
        });

        Self {
            container,
            url_bar,
            web_view,
        }
    }

    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    pub fn load_url(&self, url: &str) {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            self.web_view.load_uri(&format!("https://{}", url));
        } else {
            self.web_view.load_uri(url);
        }
        self.url_bar.set_text(url);
    }

    pub fn reload(&self) {
        self.web_view.reload();
    }
} 