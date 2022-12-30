use adw::{prelude::WidgetExt, subclass::prelude::AdwApplicationImpl, ColorScheme, StyleManager};
use glib::subclass::{prelude::{ObjectImpl, ObjectImplExt}, types::ObjectSubclass};
use gtk::{
    subclass::{
        prelude::{ApplicationImpl, GtkApplicationImpl},
        widget::WidgetImpl,
    },
    gio::{File, prelude::FileExt},
    gdk::Display,
    CssProvider,
    StyleContext,
    STYLE_PROVIDER_PRIORITY_APPLICATION, traits::GtkApplicationExt
};
use std::{sync::{Arc, Mutex}, cell::RefCell};
use super::data::ApplicationData;
use crate::{ui::main_window::MainWindow, simulator::Simulator};

#[derive(Default)]
pub struct ApplicationTemplate {
    data: Arc<Mutex<ApplicationData>>,
    simulator: RefCell<Option<Simulator>>
}

impl ApplicationTemplate {
    const CSS_RESOURCE: &'static str = "/style/style.css";

    fn start_simulation(&self) {
        *self.simulator.borrow_mut() = Some(Simulator::new(self.data.clone()))
    }

    fn stop_simulation(&self) {
        if let Some(simulator) = self.simulator.replace(None) {
            simulator.join();
        }
    }

    fn create_window(&self, application: &super::Application) {
        StyleManager::default().set_color_scheme(ColorScheme::ForceDark);

        let provider = CssProvider::new();
        provider.load_from_resource(Self::CSS_RESOURCE);
        // We give the CssProvided to the default screen so the CSS rules we added
        // can be applied to our window.
        StyleContext::add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // build the application window and UI
        let window = MainWindow::new(application, self.data.clone());
        window.show();
    }

    pub fn save_as(&self) -> Result<(), String> {
        Err("save_as is not implemented()".to_string())
    }

    pub fn save(&self) {
        let data = self.data.lock().unwrap();
        let res = {
            match data.file() {
                Some(_) => data.save(),
                None => self.save_as(),
            }
        };

        if let Err(err) = res {
            crate::die(err.as_str())
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ApplicationTemplate {
    const NAME: &'static str = "Application";
    type Type = super::Application;
    type ParentType = adw::Application;
}

impl ObjectImpl for ApplicationTemplate {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        obj.setup_gactions();
        obj.set_accels_for_action("app.quit", &["<primary>Q", "<primary>W"]);
        obj.set_accels_for_action("app.about", &["<primary>comma"]);
        obj.set_accels_for_action("app.save", &["<primary>S"]);
        obj.set_accels_for_action("app.save-as", &["<primary><shift>S"]);
    }
}
impl ApplicationImpl for ApplicationTemplate {
    fn activate(&self, application: &Self::Type) {
        self.create_window(application);
        self.start_simulation();
    }

    fn open(&self, application: &Self::Type, files: &[File], _hint: &str) {
        assert!(files.len() != 0);

        let file = &files[0];
        if file.path().is_none() {
            crate::die("File path is None");
        }

        let data = ApplicationData::build(file.to_owned());
        if let Err(err) = data {
            crate::die(err.as_str());
        }

        let mut old_data = self.data.lock().unwrap();
        *old_data = data.unwrap();
        std::mem::drop(old_data);

        self.create_window(application);
    }

    fn shutdown(&self, _application: &Self::Type) {
        self.stop_simulation();
        self.save();
    }
}
impl GtkApplicationImpl for ApplicationTemplate {}
impl AdwApplicationImpl for ApplicationTemplate {}
impl WidgetImpl for ApplicationTemplate {}
