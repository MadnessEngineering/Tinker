use gtk::prelude::*;
use gtk::{self, Box as GtkBox, Label, PopoverMenu, Orientation, gio, GestureClick};

pub struct TabLabel {
    container: GtkBox,
    label: Label,
    popover: PopoverMenu,
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

        // Create menu model
        let menu_model = gio::Menu::new();
        menu_model.append(Some("New Tab"), Some("tab.new"));
        menu_model.append(Some("Reload"), Some("tab.reload"));
        menu_model.append(Some("Duplicate"), Some("tab.duplicate"));
        menu_model.append(Some("Close"), Some("tab.close"));

        // Create popover menu
        let popover = PopoverMenu::from_model(Some(&menu_model));
        container.add_controller(&popover);

        // Add right-click gesture
        let gesture = GestureClick::builder()
            .button(3) // Right mouse button
            .build();
        
        let popover_clone = popover.clone();
        gesture.connect_pressed(move |gesture, _, x, y| {
            popover_clone.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
            popover_clone.popup();
        });
        container.add_controller(&gesture);

        // Add widgets to container
        container.append(&label);

        Self {
            container,
            label,
            popover,
        }
    }

    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    pub fn set_title(&self, title: &str) {
        self.label.set_text(title);
    }
} 