use gtk::prelude::*;
use gtk::*;
use sourceview::*;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process;
use std::rc::Rc;
use std::sync::{Arc, RwLock, Mutex};
// use std::sync::atomic::{AtomicBool, Ordering};
use std::fs::{self, DirEntry};

use gettextrs::gettext;

use crate::state::{ActiveMetadata};

use crate::config::{APP_ID, PROFILE};
use crate::window_state;

// use glib::{clone, Bytes, GString, MainContext, Receiver, SpawnFlags};
use gio::{ActionMapExt,
    // ApplicationExt, SimpleAction
};

pub struct TabNotebook {
    pub notebook: gtk::Notebook,
    pub tabs: Vec<gtk::Box>,
}

pub struct EditView {
    pub container: ScrolledWindow,
    pub view: sourceview::View,
    pub buff: sourceview::Buffer,
}

impl TabNotebook {
    pub fn new() -> TabNotebook {
        TabNotebook {
            notebook: gtk::Notebook::new(),
            tabs: Vec::new(),
        }
    }

    pub fn create_notebook_tab(&mut self, title: &str, widget: Widget) -> u32 {
        let close_image = gtk::Image::from_icon_name(Some("window-close"), IconSize::Button.into());
        let button = gtk::Button::new();
        let label = gtk::Label::new(Some(title));
        let tab = gtk::Box::new(Orientation::Horizontal, 0);

        button.set_relief(ReliefStyle::None);
        button.set_focus_on_click(false);
        button.add(&close_image);

        tab.pack_start(&label, false, false, 0);
        tab.pack_start(&button, false, false, 0);
        tab.show_all();

        let index = self.notebook.append_page(&widget, Some(&tab));
        self.tabs.push(tab);

        index
    }
}

impl EditView {
    pub fn new() -> EditView {
        let buff = Buffer::new(None::<&gtk::TextTagTable>);
        let view = View::new_with_buffer(&buff);
        let container = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        container.add(&view);

        configure_source_view(&view, &buff);

        EditView {container, view, buff}
    }
}

fn configure_source_view(view: &View, buff: &Buffer) {

    // WidgetExt::override_font(view, &FontDescription::from_string("monospace"));

    LanguageManager::new()
    .get_language("markdown")
    .map(|markdown| buff.set_language(Some(&markdown)));

    let manager = StyleSchemeManager::new();
    manager
    .get_scheme("Builder")
    .or(manager.get_scheme("Classic"))
    .map(|theme| buff.set_style_scheme(Some(&theme)));

    view.set_show_line_numbers(true);
    // view.set_monospace(true);
    view.set_insert_spaces_instead_of_tabs(true);
    view.set_indent_width(4);
    // view.set_smart_backspace(true);
    view.set_right_margin(100);
    view.set_left_margin(10);
    view.set_show_right_margin(true);
    // view.set_background_pattern(BackgroundPatternType::Grid);
}

pub struct MainWin {
    pub widget: gtk::ApplicationWindow,
    settings: gio::Settings,
    // pub headerbar: gtk::HeaderBar,
    // browser
    pub treeview: gtk::TreeView,
    pub treestore: gtk::TreeStore,
    pub liststore: gtk::ListStore,
    // notebook & editview
    pub notebook: gtk::Notebook,
    pub tabs: Vec<gtk::Box>,
    pub editview: EditView,
}

impl MainWin {
    pub fn new(app: &gtk::Application) -> Rc<Self> {
        let settings = gio::Settings::new(APP_ID);

        let builder = gtk::Builder::from_resource("/me/idlesong/cuteedit/window.ui");
        get_widget!(builder, gtk::ApplicationWindow, window);
        get_widget!(builder, gtk::TreeView, treeview);
        get_widget!(builder, gtk::Notebook, notebook);
        // let tab_notebook = TabNotebook::new();

        // file browser config
        let treestore = gtk::TreeStore::new(&[String::static_type()]);
        treeview.set_model(Some(&treestore));
        treeview.set_headers_visible(false);

        // append text column
        let column = gtk::TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();
        column.pack_start(&cell, true);
        column.add_attribute(&cell, "text", 0);
        treeview.append_column(&column);

        // create liststore for search_dialog
        let data = [ "France".to_string(), "Italy".to_string(), "Sweden".to_string(),
                     "Switzerland".to_string(), "Shanghai".to_string()];

        let liststore = gtk::ListStore::new(&[String::static_type()]);

        let col_indices: [u32; 1] = [0];
        for d in data.iter() {
            let values: [&dyn ToValue; 1] = [&d];
            liststore.set(&liststore.append(), &col_indices, &values);
        }

        // notebook config
        let tabs = Vec::new();

        // editview config
        let editview = EditView::new();

        let main_win = Rc::new(MainWin {
            widget: window,
            settings,
            // headerbar,
            treeview,
            treestore,
            liststore,
            notebook,
            tabs,
            editview,
        });

        main_win.setup_gactions(app);

        // open file
        let current_file = Arc::new(RwLock::new(None));
        {
            let open_action = gio::SimpleAction::new("open", None);
            open_action.connect_activate(clone!(@weak main_win => @default-panic, move |_, _| {
                trace!("Handling action: 'open' new file");

                main_win.handle_open_file(current_file.clone());
            }));
            app.add_action(&open_action);
        }

        // open dir folder
        let current_file = Arc::new(RwLock::new(None));
        {
            let open_action = gio::SimpleAction::new("open_folder", None);
            open_action.connect_activate(clone!(@weak main_win => @default-panic, move |_, _| {
                trace!("Handling action: 'open' folder");

                main_win.handle_open_folder(current_file.clone());
            }));
            app.add_action(&open_action);
        }

        // open folder/dir file
        let current_file = Arc::new(RwLock::new(None));
        {
            main_win.handle_open_file_from_browser(current_file.clone());
        }


        main_win.init();
        main_win
    }

    fn setup_gactions(&self, app: &gtk::Application) {
        // Below here we connect all actions, meaning that these closures will be run when the respective
        // action is triggered (e.g. by a button press)

        // let current_file = Arc::new(RwLock::new(None));
        //
        // // open file
        // {
        //     let open_action = gio::SimpleAction::new("open", None);
        //     open_action.connect_activate(clone!(@weak self as win => @default-panic, move |_, _| {
        //         trace!("Handling action: 'open' new file");
        //
        //         win.handle_open_button(current_file.clone());
        //     }));
        //     app.add_action(&open_action);
        // }

        // {
        //     let open_action = SimpleAction::new("open", None);
        //     open_action.connect_activate(clone!(@weak main_win => @default-panic, move |_,_| {
        //         trace!("Handling action: 'open'");
        //         main_win.handle_open_button();
        //     }));
        //     app.add_action(&open_action);
        // }

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

    pub fn create_notebook_tab(&mut self, title: &str, widget: Widget) -> u32 {
        let close_image = gtk::Image::from_icon_name(Some("window-close"), IconSize::Button.into());
        let button = gtk::Button::new();
        let label = gtk::Label::new(Some(title));
        let tab = gtk::Box::new(Orientation::Horizontal, 0);

        button.set_relief(ReliefStyle::None);
        button.set_focus_on_click(false);
        button.add(&close_image);

        tab.pack_start(&label, false, false, 0);
        tab.pack_start(&button, false, false, 0);
        tab.show_all();

        let index = self.notebook.append_page(&widget, Some(&tab));
        self.tabs.push(tab);

        index
    }

    fn handle_open_file(&self, current_file: Arc<RwLock<Option<ActiveMetadata>>>){
        let notebook = self.notebook.clone();
        let tabs = self.tabs.clone();

        let fcn = FileChooserNative::new(
            Some(gettext("Open a file to edit").as_str()),
            Some(&self.widget),
            FileChooserAction::Open,
            Some(gettext("Open").as_str()),
            Some(gettext("Cancel").as_str()),
        );
        fcn.set_transient_for(Some(&self.widget.clone()));
        fcn.set_select_multiple(true);

        {
            let lock = current_file.read().unwrap();
            let file_path: Option<PathBuf>;
            if let Some(ref path) = * lock {
                file_path = path.get_dir();
            } else {
                file_path = None;
            }
            file_path.map(|p| fcn.set_current_folder(p));
        }

        fcn.connect_response( move |fcd, res| {
            debug!("FileChooserNative open response: '{:#?}'", res);

            if res == ResponseType::Accept {
                for filename in fcd.get_filenames() {
                    let new_file = filename.to_string_lossy().into_owned();
                    // match std::fs::File::open(&file_str) {
                    //     Ok(_) => main_win.req_new_view(Some(file_str)),
                    //     Err(e) => {
                    //         let err_msg = format!("{} '{}': {}", &gettext("Couldn’t open file"), &file_str, &e.to_string());
                    //         ErrorDialog::new(ErrorMsg{msg: err_msg, fatal: false});
                    //     }
                    // }
                    if let Ok(mut file) = File::open(&new_file) {
                        let mut contents = String::new();
                        let _ = file.read_to_string(&mut contents);

                        // set_title(&headerbar, &new_file);
                        // if let Some(parent) = new_file.parent() {
                        //     let subtitle: &str = &parent.to_string_lossy();
                        //     headerbar.set_subtitle(subtitle);
                        // }

                        // if let Some(filename) = new_file.file_name() {
                            // let title: &str = &filename.to_string_lossy();
                            // let label = gtk::Label::new(Some(&new_file));
                            debug!("Tab title: '{:#?}'", new_file);
                            let label = gtk::Label::new(Some(&new_file));

                            *current_file.write().unwrap() =
                                Some(ActiveMetadata::new(filename, &contents.as_bytes()));

                            // create tab, notebook append_page, show_all
                            let close_image = gtk::Image::from_icon_name(Some("window-close"), IconSize::Button.into());
                            let button = gtk::Button::new();
                            button.set_relief(ReliefStyle::None);
                            button.set_focus_on_click(false);
                            button.add(&close_image);

                            let tab = gtk::Box::new(Orientation::Horizontal, 0);
                            tab.pack_start(&label, false, false, 0);
                            tab.pack_start(&button, false, false, 0);
                            tab.show_all();

                            let edit_view = EditView::new();

                            edit_view.buff.set_text(&contents);

                            let index = notebook.append_page(&edit_view.container, Some(&tab));
                            notebook.show_all();

                            // tabs.push(tab);
                            // let notebook_weak = notebook.downgrade();
                            // button.connect_clicked(move |_| {
                            //     let notebook = upgrade_weak!(notebook_weak);
                            //     let index = notebook.page_num(&source.container)
                            //                         .expect("Couldn't get page_num from notebook");
                            //     notebook.remove_page(Some(index));
                            // });
                        // }
                    }
                }
            }
        });

        // self.saving.replace(true);
        fcn.run();
    }

    fn handle_open_file_from_browser(&self, current_file: Arc<RwLock<Option<ActiveMetadata>>>){
        // let headerbar = self.header.container.clone();
        let notebook = self.notebook.clone();

        let treeview = self.treeview.clone();
        let left_selection = treeview.get_selection();

        left_selection.connect_changed(move |tree_selection| {
            let (left_model, iter) = tree_selection.get_selected().expect("Couldn't get selected");
            let mut value = left_model.get_value(&iter, 0).get::<String>().expect("Couldn't get value");

            let new_file = Path::new(&value.unwrap()).to_path_buf();
            println!("new open file path{:?}", new_file);

            if new_file.exists() {
                debug!("file path exists{:?}", new_file);
                if let Ok(mut file) = File::open(&new_file) {
                    let mut contents = String::new();
                    let _ = file.read_to_string(&mut contents);

                    // set_title(&headerbar, &new_file);
                    if let Some(parent) = new_file.parent() {
                        let subtitle: &str = &parent.to_string_lossy();
                        // headerbar.set_subtitle(subtitle);
                    }

                    if let Some(filename) = new_file.file_name() {
                        let title: &str = &filename.to_string_lossy();
                        let label = gtk::Label::new(Some(title));

                        *current_file.write().unwrap() =
                        Some(ActiveMetadata::new(new_file, &contents.as_bytes()));

                        // create tab, notebook append_page, show_all
                        let close_image = gtk::Image::from_icon_name(Some("window-close"), IconSize::Button.into());
                        let button = gtk::Button::new();
                        button.set_relief(ReliefStyle::None);
                        button.set_focus_on_click(false);
                        button.add(&close_image);

                        let tab = gtk::Box::new(Orientation::Horizontal, 0);
                        tab.pack_start(&label, false, false, 0);
                        tab.pack_start(&button, false, false, 0);
                        tab.show_all();

                        let edit_view = EditView::new();

                        edit_view.buff.set_text(&contents);

                        let index = notebook.append_page(&edit_view.container, Some(&tab));
                        notebook.show_all();

                        // let notebook_weak = notebook.downgrade();
                        // button.connect_clicked(move |_| {
                        //     let notebook = upgrade_weak!(notebook_weak);
                        //     let index = notebook.page_num(&source.container)
                        //     .expect("Couldn't get page_num from notebook");
                        //     notebook.remove_page(Some(index));
                        // });
                    }
                    // editor.set_text(&contents);
                    // preview.load_html(&render(&contents), None);
                } else {
                    println!("invalid path selected!");
                };
            }
        });
    }

    fn handle_open_folder(&self, current_file: Arc<RwLock<Option<ActiveMetadata>>>){
        let treeview = self.treeview.clone();
        let treestore = self.treestore.clone();
        let liststore = self.liststore.clone();
        // treeview.set_headers_visible(false);

        let fcn = FileChooserNative::new(
            Some(gettext("Open a folder").as_str()),
            Some(&self.widget),
            FileChooserAction::SelectFolder,
            Some(gettext("Open Folder").as_str()),
            Some(gettext("Cancel").as_str()),
        );
        fcn.set_transient_for(Some(&self.widget.clone()));
        fcn.set_select_multiple(true);

        let lock = current_file.read().unwrap();
        let file_path: Option<PathBuf>;
        if let Some(ref path) = * lock {
            file_path = path.get_dir();
        } else {
            file_path = None;
        }

        file_path.map(|p| fcn.set_current_folder(p));

        fcn.connect_response( move |fcd, res| {
            debug!("FileChooserNative open response: '{:#?}'", res);

            let mut vec: Vec<PathBuf> = Vec::new();
            if res == ResponseType::Accept {
                // let mut vec_files: Vec<PathBuf> = Vec::new();
                treestore.clear();
                liststore.clear();

                if let Some(dir) = fcd.get_filename(){
                    let folder_pathbuf = dir.clone();
                    let project_dir = dir.clone();
                    if let Some(parent_dir) = dir.parent().clone() {
                        if let Ok(folder_dir) = folder_pathbuf.as_path().strip_prefix(&parent_dir){
                            debug!("folder_dir:'{:#?}'", folder_dir);
                            let dir_name: [&dyn ToValue; 1] = [&folder_dir.to_str()];
                            let iter = treestore.insert_with_values(None, None, &[0], &dir_name);

                            if let Ok(entries) = fs::read_dir(dir){
                                for entry in entries{
                                    if let Ok(entry) = entry {
                                        // debug!("entry in entries:'{:#?}'", entry);

                                        // insert dir shortname to treestore
                                        if let Ok(file_path) = entry.path().as_path().strip_prefix(&project_dir.as_path()){
                                            let dir_name: [&dyn ToValue; 1] = [&file_path.to_str()];
                                            treestore.insert_with_values(Some(&iter), None, &[0], &dir_name);
                                            // liststore.set(&liststore.append(), &col_indices, &dir_name);
                                        }

                                        // if it's dir, also add sub_dir files to treestore
                                        let entry_path = entry.path();
                                        if entry_path.is_dir() {
                                            let sub_dir = entry_path.clone();
                                            if let Ok(sub_entries) = fs::read_dir(entry_path){
                                                for sub_entry in sub_entries {
                                                    if let Ok(sub_dir_file) = sub_entry {
                                                        if let Ok(file_path) = sub_dir_file.path().as_path().strip_prefix(&sub_dir.as_path()){
                                                            let dir_name: [&dyn ToValue; 1] = [&file_path.to_str()];
                                                            // treestore.insert_with_values(Some(&iter), Some(1), &[0], &dir_name);
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                    }
                                }
                                // vec.append(&mut vec_files);
                            }
                        }
                    }
                    treeview.expand_to_path(&TreePath::new_first());
                }
            }
        });
        fcn.run();
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

/// An Extension trait for `MainWin`. This is implemented for `Rc<MainWin>`, allowing for a nicer
/// API (where we can do stuff like `self.close()` instead of `Self::close(main_win)`).
pub trait MainWinExt {
    // fn handle_open_button(&self);
}

// impl
