use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Label, Orientation};

pub struct Tab {
    container: GtkBox,
    content: GtkBox,
    title: String,
}

impl Tab {
    pub fn new(title: &str) -> Self {
        // Create the main container
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();

        // Create the content area
        let content = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .vexpand(true)
            .hexpand(true)
            .build();

        // Add the content area to the container
        container.append(&content);

        Self {
            container,
            content,
            title: title.to_string(),
        }
    }

    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    pub fn content(&self) -> &GtkBox {
        &self.content
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }
}

pub struct TabLabel {
    container: GtkBox,
    label: Label,
    close_button: Button,
}

impl TabLabel {
    pub fn new(title: &str) -> Self {
        // Create the container
        let container = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();

        // Create the label
        let label = Label::builder()
            .label(title)
            .build();

        // Create the close button
        let close_button = Button::builder()
            .label("×")
            .build();

        // Add widgets to container
        container.append(&label);
        container.append(&close_button);

        Self {
            container,
            label,
            close_button,
        }
    }

    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    pub fn set_title(&self, title: &str) {
        self.label.set_text(title);
    }

    pub fn connect_close<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.close_button.connect_clicked(move |_| {
            f();
        });
    }
} 