use gtk::prelude::*;
use gtk::{Box as GtkBox, ScrolledWindow, Image, Orientation};
use webkit::prelude::*;
use webkit::WebView as WebKitView;
use sourceview5::{View as SourceView, Buffer as SourceBuffer};
use std::path::Path;
use url::Url;

pub enum ContentType {
    Web,
    Text,
    Code,
    Image,
}

pub struct Content {
    container: GtkBox,
    content_type: ContentType,
    web_view: Option<WebKitView>,
    source_view: Option<SourceView>,
    image_view: Option<Image>,
}

impl Content {
    pub fn new(content_type: ContentType) -> Self {
        // Create main container
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .vexpand(true)
            .hexpand(true)
            .build();

        // Create scrolled window for content
        let scrolled = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();
        container.append(&scrolled);

        // Initialize content based on type
        let mut web_view = None;
        let mut source_view = None;
        let mut image_view = None;

        match content_type {
            ContentType::Web => {
                let view = WebKitView::new();
                scrolled.set_child(Some(&view));
                web_view = Some(view);
            }
            ContentType::Text | ContentType::Code => {
                let buffer = SourceBuffer::new(None);
                let view = SourceView::with_buffer(&buffer);
                view.set_monospace(true);
                view.set_show_line_numbers(true);
                view.set_highlight_current_line(true);
                scrolled.set_child(Some(&view));
                source_view = Some(view);
            }
            ContentType::Image => {
                let view = Image::new();
                scrolled.set_child(Some(&view));
                image_view = Some(view);
            }
        }

        Self {
            container,
            content_type,
            web_view,
            source_view,
            image_view,
        }
    }

    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    pub fn load_content(&self, source: &str) -> Result<(), String> {
        match self.content_type {
            ContentType::Web => {
                if let Some(view) = &self.web_view {
                    if let Ok(url) = Url::parse(source) {
                        view.load_uri(url.as_str());
                        Ok(())
                    } else {
                        Err("Invalid URL".to_string())
                    }
                } else {
                    Err("WebView not initialized".to_string())
                }
            }
            ContentType::Text | ContentType::Code => {
                if let Some(view) = &self.source_view {
                    if let Ok(text) = std::fs::read_to_string(source) {
                        view.buffer().set_text(&text);
                        Ok(())
                    } else {
                        Err("Failed to read file".to_string())
                    }
                } else {
                    Err("SourceView not initialized".to_string())
                }
            }
            ContentType::Image => {
                if let Some(view) = &self.image_view {
                    if Path::new(source).exists() {
                        view.set_filename(Some(source));
                        Ok(())
                    } else {
                        Err("Image file not found".to_string())
                    }
                } else {
                    Err("ImageView not initialized".to_string())
                }
            }
        }
    }

    pub fn set_text_content(&self, text: &str) -> Result<(), String> {
        if let Some(view) = &self.source_view {
            view.buffer().set_text(text);
            Ok(())
        } else {
            Err("SourceView not initialized".to_string())
        }
    }

    pub fn get_text_content(&self) -> Option<String> {
        self.source_view.as_ref().map(|view| {
            let buffer = view.buffer();
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            buffer.text(&start, &end, false).to_string()
        })
    }

    pub fn set_editable(&self, editable: bool) {
        if let Some(view) = &self.source_view {
            view.set_editable(editable);
        }
    }

    pub fn set_language(&self, language: Option<&sourceview5::Language>) {
        if let Some(view) = &self.source_view {
            if let Some(buffer) = view.buffer().downcast_ref::<sourceview5::Buffer>() {
                buffer.set_language(language);
            }
        }
    }
} 