use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use gtk::{gio, glib, prelude::*, subclass::prelude::*};

use crate::{
    application::{editor::EditorMode, Application},
    simulator::PlotProvider,
};

use super::circuit_view::CircuitView;

glib::wrapper! {
    pub struct CircuitPanel(ObjectSubclass<CircuitPanelTemplate>)
        @extends gtk::Box, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl CircuitPanel {
    pub fn new(app: Application) -> Self {
        let panel: Self = glib::Object::new::<Self>(&[]);
        panel.imp().set_title(app.imp().file_name().as_str());
        panel.set_application(app);
        panel
    }

    pub fn reset_ui(&self) {
        self.imp().close_tabs();
        self.undo_button().set_sensitive(false);
        self.redo_button().set_sensitive(false);
    }

    #[template_callback]
    pub fn undo_latest(&self, _btn: &gtk::Button) {
        self.imp().application.borrow().undo_action();
    }

    #[template_callback]
    pub fn redo_latest(&self, _btn: &gtk::Button) {
        self.imp().application.borrow().redo_action();
    }

    pub fn undo_button(&self) -> &gtk::Button {
        &self.imp().undo_button
    }

    pub fn redo_button(&self) -> &gtk::Button {
        &self.imp().redo_button
    }

    pub fn open_tab(&self, plot_provider: PlotProvider) {
        if let PlotProvider::Module(_, module_name) = &plot_provider {
            let mut i = 0;
            let view = &self.imp().view;
            while i < view.n_pages() {
                let page = view.nth_page(i);
                if page.title().eq(module_name) {
                    self.imp().view.set_selected_page(&page);
                    return;
                }
                i += 1;
            }

            // page not found, create new
            self.imp().new_tab(module_name, plot_provider.clone());
        }
    }

    pub fn set_application(&self, app: Application) {
        self.imp().application.replace(app);
    }

    pub fn new_tab(&self, title: &str, plot_provider: PlotProvider) {
        self.imp().new_tab(title, plot_provider)
    }

    pub fn set_title(&self, title: &str) {
        self.imp().set_title(title)
    }

    pub fn remove_tab(&self, module_name: &String) {
        self.imp().remove_tab(module_name)
    }

    pub fn push_error(&self, error: String) {
        let template = self.imp();
        if template.info_bar.is_visible() {
            let mut errors = template.errors.borrow_mut();
            if !errors.contains(&error) {
                errors.push(error);
            }
        } else {
            template.info_label.set_label(&error);
            template.info_bar.show();
        }
    }
}

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/content/circuit-panel.ui")]
pub struct CircuitPanelTemplate {
    #[template_child]
    pub header_bar: TemplateChild<adw::HeaderBar>,

    #[template_child]
    pub back_button: TemplateChild<gtk::Button>,

    #[template_child]
    pub view: TemplateChild<adw::TabView>,

    #[template_child]
    pub tab_bar: TemplateChild<adw::TabBar>,

    #[template_child]
    undo_button: TemplateChild<gtk::Button>,

    #[template_child]
    redo_button: TemplateChild<gtk::Button>,

    #[template_child]
    toggle_grid_button: TemplateChild<gtk::ToggleButton>,

    #[template_child]
    info_bar: TemplateChild<gtk::InfoBar>,

    #[template_child]
    info_label: TemplateChild<gtk::Label>,

    #[template_child]
    info_close_button: TemplateChild<gtk::Button>,

    application: RefCell<Application>,
    pages: RefCell<HashMap<String, adw::TabPage>>,
    force_closing: Cell<bool>,
    errors: RefCell<Vec<String>>,
}

impl CircuitPanelTemplate {
    fn add_page(&self, content: &CircuitView, title: &str) -> adw::TabPage {
        let page = self.view.add_page(content, None);
        page.set_indicator_activatable(true);
        page.set_title(title);
        page
    }

    fn new_tab(&self, title: &str, plot_provider: PlotProvider) {
        let content = CircuitView::new(self.application.borrow().clone(), plot_provider);
        if self.toggle_grid_button.is_active() {
            content.set_editor_mode(EditorMode::Grid);
        }

        let page = self.add_page(&content, title);
        self.view.set_selected_page(&page);
        self.pages.borrow_mut().insert(title.to_owned(), page);
    }

    fn remove_tab(&self, module_name: &String) {
        if let Some(page) = self.pages.borrow().get(module_name) {
            self.view.close_page(page);
        }
    }

    fn set_title(&self, title: &str) {
        (self
            .header_bar
            .title_widget()
            .unwrap()
            .downcast_ref()
            .unwrap() as &adw::WindowTitle)
            .set_subtitle(title);
    }

    fn close_tabs(&self) {
        self.force_closing.set(true);
        for i in (0..self.view.n_pages()).rev() {
            self.view.close_page(&self.view.nth_page(i));
        }
        self.force_closing.set(false);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for CircuitPanelTemplate {
    const NAME: &'static str = "CircuitPanel";
    type Type = CircuitPanel;
    type ParentType = gtk::Box;

    fn class_init(class: &mut Self::Class) {
        class.bind_template();
        class.bind_template_instance_callbacks();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for CircuitPanelTemplate {
    fn constructed(&self) {
        self.parent_constructed();
        self.toggle_grid_button.connect_toggled(glib::clone!(@weak self as widget => move |btn| {
            let mut i = 0;
            while i < widget.view.n_pages() && let Ok(circuit_view) = widget.view.nth_page(i).child().downcast::<CircuitView>() {
                circuit_view.set_editor_mode(EditorMode::from(btn.is_active()));
                if widget.view.nth_page(i).is_selected() {
                    circuit_view.rerender();
                }
                i += 1;
            }
        }));

        self.view.connect_close_page(glib::clone!(@weak self as widget => @default-return false, move |view, page| {
            let is_main = page.child().downcast::<CircuitView>()
                .map(|circuit_view| circuit_view.plot_provider().is_main());
            view.close_page_finish(page, !matches!(is_main, Ok(true)) || widget.force_closing.get());
            true
        }));

        self.info_close_button
            .connect_clicked(glib::clone!(@weak self as widget => move |_| widget.info_bar.hide()));
        self.info_bar
            .connect_hide(glib::clone!(@weak self as widget => move |bar| {
                let mut errors = widget.errors.borrow_mut();
                if let Some(err) = errors.pop() {
                    widget.info_label.set_label(&err);
                    bar.show();
                }
            }));
    }
}

impl WidgetImpl for CircuitPanelTemplate {}

impl BoxImpl for CircuitPanelTemplate {}
