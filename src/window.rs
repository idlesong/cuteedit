use gtk::prelude::*;

use crate::config::{APP_ID, PROFILE};
use crate::window_state;

use gio::{ActionMapExt,
    // ApplicationExt, SimpleAction
};

pub struct Window {
    pub widget: gtk::ApplicationWindow,
    settings: gio::Settings,
}

impl Window {
    pub fn new(app: &gtk::Application) -> Self {
        let settings = gio::Settings::new(APP_ID);

        let builder = gtk::Builder::from_resource("/me/idlesong/cuteedit/window.ui");
        get_widget!(builder, gtk::ApplicationWindow, window);

        let window_widget = Window {
            widget: window,
            settings,
        };

        window_widget.setup_gactions(app);

        window_widget.init();
        window_widget
    }

    fn setup_gactions(&self, app: &gtk::Application) {
        // Below here we connect all actions, meaning that these closures will be run when the respective
        // action is triggered (e.g. by a button press)

        // open file
        {
            let open_action = gio::SimpleAction::new("open", None);
            open_action.connect_activate(clone!(@weak self.widget as window => move |_,_| {
                trace!("Handling action: 'open' new file");
                // window.handle_open_button();
                let builder = gtk::Builder::from_resource("/me/idlesong/cuteedit/filechooser_dialog.ui");
                get_widget!(builder, gtk::FileChooserDialog, filechooser_dialog);
                filechooser_dialog.set_transient_for(Some(&window));

                filechooser_dialog.connect_response(|dialog, _| dialog.close());
                filechooser_dialog.show();
            }));
            app.add_action(&open_action);
        }

        // action!(
        //     app,
        //     "open",
        //     clone!(@weak self.widget as window => move |_, _| {
        //         trace!("Handling action: 'open' file");
        //
        //         let builder = gtk::Builder::from_resource("/me/idlesong/cuteedit/filechooser_dialog.ui");
        //         get_widget!(builder, gtk::FileChooserDialog, filechooser_dialog);
        //         filechooser_dialog.set_transient_for(Some(&window));
        //
        //         filechooser_dialog.connect_response(|dialog, _| dialog.close());
        //         filechooser_dialog.show();
        //     })
        // );
    }

    fn init(&self) {
        // Devel Profile
        if PROFILE == "Devel" {
            self.widget.get_style_context().add_class("devel");
        }

        // load latest window state
        window_state::load(&self.widget, &self.settings);

        // save window state on delete event
        self.widget.connect_delete_event(
            clone!(@strong self.settings as settings => move |window, _| {
                if let Err(err) = window_state::save(&window, &settings) {
                    warn!("Failed to save window state, {}", err);
                }
                Inhibit(false)
            }),
        );
    }
}
