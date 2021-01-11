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
use std::env;

use gettextrs::gettext;

use gdk_pixbuf::Pixbuf;

use crate::state::{ActiveMetadata};

use crate::config::{APP_ID, PROFILE};
use crate::window_state;

// use glib::{clone, Bytes, GString, MainContext, Receiver, SpawnFlags};
use gio::{ActionMapExt,
    // ApplicationExt, SimpleAction
};

pub struct EditView {
    pub container: ScrolledWindow,
    pub view: sourceview::View,
    pub buff: sourceview::Buffer,
    // pub main_state: Rc<RefCell<MainState>>,
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
        let treestore = gtk::TreeStore::new(&[String::static_type(), String::static_type()]);
        treeview.set_model(Some(&treestore));
        treeview.set_headers_visible(false);

        // append text column
        let column = gtk::TreeViewColumn::new();
        // let cell = gtk::CellRendererText::new();
        // column.pack_start(&cell, true);
        // column.add_attribute(&cell, "text", 0);

        // let cell = gtk::CellRendererPixbuf::new();
        // column.pack_start(&cell, true);
        // column.add_attribute(&cell, "pixbuf", 0);

        // column of cell2
        let cell2 = gtk::CellRendererText::new();
        column.pack_start(&cell2, true);
        column.add_attribute(&cell2, "text", 0);


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
            // main_win.handle_open_file_from_browser(current_file.clone());
            main_win.on_treeview_node_clicked(current_file.clone());
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

    // fn on_treeview_node_clicked() {
    // selection.connect_changed()
    // on_toggle_row()
    // treeview.connect_activate(){
    //}
    //}
    fn on_treeview_row_activate(){
        // read children dir.
    }

    fn on_treeview_node_clicked(&self, current_file: Arc<RwLock<Option<ActiveMetadata>>>){
        // let headerbar = self.header.container.clone();
        let notebook = self.notebook.clone();

        let treeview = self.treeview.clone();
        let left_selection = treeview.get_selection();

        left_selection.connect_changed(move |tree_selection| {
            let (left_model, mut iter) = tree_selection.get_selected().expect("Couldn't get selected");
            let mut value = left_model.get_value(&iter, 0).get::<String>().expect("Couldn't get value");

            // let mut node_path = left_model.get_path(&iter).expect("Couldn't get path");
            let file_name_str = &value.unwrap();

            // build selected node's full_path first, note to remove project dir
            let mut path_str = String::from("");
            let mut dir_fullpath = file_name_str.clone(); //String::from("");

            // debug!("iter start:{:#?}", left_model.get_string_from_iter(&iter));
            if let Some(iter) = left_model.iter_parent(&iter){
                let mut path_iter = Some(iter.clone());

                loop {
                    match path_iter {
                        Some(iter) => {
                            path_iter = left_model.iter_parent(&iter);

                            path_str = left_model.get_value(&iter, 0).get::<String>().expect("Couldn't get value").unwrap();
                            dir_fullpath = [[&path_str,"/"].concat(), dir_fullpath].concat();
                            debug!("dir_fullpath:{:#?}", dir_fullpath);
                        },
                        _ => {break;}
                    }
                }
            }

            // remove project dir
            let mut selected_path_buf = Path::new(&dir_fullpath).to_path_buf();
            let mut node_full_path = Path::new(&dir_fullpath);

            if let Ok(node_full_path) = node_full_path.strip_prefix([&path_str,"/"].concat()){
                selected_path_buf = node_full_path.to_path_buf();
            }
            trace!("selected node path_buf:{:#?}", selected_path_buf);

            // If selected node is (1)dir, toggle treeview row. (2)file, open file
            // open the file, if it's a file
            if selected_path_buf.is_file() {
                debug!("file:{:#?} selected!", selected_path_buf);
                if let Ok(mut file) = File::open(&selected_path_buf) {
                    debug!("open file successful!");
                    let mut contents = String::new();
                    let _ = file.read_to_string(&mut contents);

                    // set_title(&headerbar, &selected_path_buf);
                    if let Some(parent) = selected_path_buf.parent() {
                        let subtitle: &str = &parent.to_string_lossy();
                        // headerbar.set_subtitle(subtitle);
                    }

                    if let Some(filename) = selected_path_buf.file_name() {
                        trace!("filename{:?}", filename);
                        let title: &str = &filename.to_string_lossy();
                        let label = gtk::Label::new(Some(title));

                        *current_file.write().unwrap() =
                            Some(ActiveMetadata::new(selected_path_buf, &contents.as_bytes()));

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
                }
            } else if selected_path_buf.is_dir() {
                // toggle_folder_view, if it's a dir; Or nothing, if it's a invalid path
                debug!("folder:{:#?} selected!", selected_path_buf);
            } else {
                trace!("invalid path selected!");
            }
        });
    }

    fn handle_open_file_from_browser(&self, current_file: Arc<RwLock<Option<ActiveMetadata>>>){
        // let headerbar = self.header.container.clone();
        let notebook = self.notebook.clone();

        let treeview = self.treeview.clone();
        let left_selection = treeview.get_selection();

        left_selection.connect_changed(move |tree_selection| {
            let (left_model, mut iter) = tree_selection.get_selected().expect("Couldn't get selected");
            let mut value = left_model.get_value(&iter, 0).get::<String>().expect("Couldn't get value");

            // let mut selected_dir_path = Path::new(&value.unwrap());
            // let mut node_path = left_model.get_path(&iter).expect("Couldn't get path");
            let file_name_str = &value.unwrap();

            let mut dir_fullpath = file_name_str.clone();

            let mut new_file = Path::new(file_name_str).to_path_buf();
            // let mut new_file = Path::new(&value.unwrap()).to_path_buf();
            // let new_file = new_file.as_path();
            // let new_file = Path::new("/home/quincy/Code/cutedit/window.ui");
            // let new_file = Path::new(&value.unwrap());
            debug!("selected dir file_name:{:#?}", new_file);
            debug!("iter start:{:#?}", left_model.get_string_from_iter(&iter));

            // TODO: If it is dir, toggle tree child. If subdir, find dir one more step.
            // TODO: build full_path from file_name, or use relative path?

            let mut path_iter = Some(iter);
            // let mut dir_fullpath = String::from(&value.expect("Couldn't get value"));
            loop {
                // path_iter = left_model.iter_parent(&iter)
                if (path_iter == None) {
                    break;
                } else {
                    iter = path_iter.expect("iter to None").clone();
                }

                path_iter = left_model.iter_parent(&iter);
                // left_model.get_value(&iter, 0).get::<String>().expect("Couldn't get value");
                dir_fullpath = left_model.get_value(&iter, 0).get::<String>().expect("Couldn't get value").unwrap();
                debug!("dir_fullpath:{:#?}", dir_fullpath);
            }


            if new_file.exists() {
                debug!("file path exists:{:#?}", new_file);
                if let Ok(mut file) = File::open(&new_file) {
                    debug!("open file successful!");
                    let mut contents = String::new();
                    let _ = file.read_to_string(&mut contents);

                    // set_title(&headerbar, &new_file);
                    if let Some(parent) = new_file.parent() {
                        let subtitle: &str = &parent.to_string_lossy();
                        // headerbar.set_subtitle(subtitle);
                    }

                    if let Some(filename) = new_file.file_name() {
                        trace!("filename{:?}", filename);
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
                    trace!("invalid path selected!");
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
                            let image = "folder/";
                            let mut folder_treepath = 0;
                            let mut subdir_folder_treepath = 0;
                            let mut position: Option<u32>;
                            let mut subdir_position: Option<u32>;
                            // let image = gtk::Image::from_icon_name(Some("window-close"), IconSize::Button.into());
                            // let image = Pixbuf::from_file("/me/idlesong/cuteedit/file.png")
                            //     .or_else(|err| {
                            //         let mut msg = err.to_string();
                            //         if err.kind() == Some(glib::FileError::Noent) {
                            //             debug!("pixbuf image file can't be found");
                            //             msg.push_str(
                            //                 "\nRelaunch this example from the same level \
                            //                  as the `resources` folder",
                            //             );
                            //         }
                            //         Err(())
                            //     })
                            //     .ok();

                            let dir_name: [&dyn ToValue; 2] = [&folder_dir.to_str(), &folder_dir.to_str()];
                            let project_iter = treestore.insert_with_values(None, None, &[0, 1], &dir_name);

                            if let Ok(entries) = fs::read_dir(dir){
                                for entry in entries{
                                    if let Ok(entry) = entry {
                                        // debug!("entry in entries:'{:#?}'", entry);

                                        // insert file shortname to treestore
                                        let mut iter = project_iter.clone();
                                        if let Ok(file_path) = entry.path().as_path().strip_prefix(&project_dir.as_path()){
                                            let dir_name: [&dyn ToValue; 2] = [&file_path.to_str(), &file_path.to_str()];

                                            // position
                                            if entry.path().is_dir(){
                                                position = Some(folder_treepath);
                                                folder_treepath = folder_treepath + 1;
                                            } else {
                                                position = None;
                                            }
                                            iter = treestore.insert_with_values(Some(&project_iter), position, &[0,1], &dir_name);
                                            // liststore.set(&liststore.append(), &col_indices, &dir_name);
                                        }

                                        // if it's dir, also add sub_dir files to treestore
                                        let entry_path = entry.path();
                                        if entry_path.is_dir() {
                                            let sub_dir = entry_path.clone();
                                            if let Ok(sub_entries) = fs::read_dir(entry_path){
                                                for sub_entry in sub_entries {
                                                    if let Ok(sub_dir_entry) = sub_entry {
                                                        if let Ok(file_path) = sub_dir_entry.path().as_path().strip_prefix(&sub_dir.as_path()){
                                                            let dir_name: [&dyn ToValue; 2] = [&file_path.to_str(), &file_path.to_str()];
                                                            // position
                                                            if sub_dir_entry.path().is_dir(){
                                                                subdir_position = Some(subdir_folder_treepath);
                                                                subdir_folder_treepath = subdir_folder_treepath + 1;
                                                            } else {
                                                                subdir_position = None;
                                                            }
                                                            treestore.insert_with_values(Some(&iter), subdir_position, &[0,1], &dir_name);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                // vec.append(&mut vec_files);
                                // debug!("treestore:'{:#?}'", treestore);
                            }
                        }
                    }
                    treeview.expand_to_path(&TreePath::new_first());
                }
            }
        });
        fcn.run();
    }

    fn handle_open_folder2(&self, current_file: Arc<RwLock<Option<ActiveMetadata>>>){
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
                            let dir_name: [&dyn ToValue; 2] = [&folder_dir.to_str(), &folder_dir.to_str()];
                            let project_iter = treestore.insert_with_values(None, None, &[0, 1], &dir_name);

                            if let Ok(entries) = fs::read_dir(dir){
                                for entry in entries{
                                    if let Ok(entry) = entry {
                                        // debug!("entry in entries:'{:#?}'", entry);

                                        // insert file shortname to treestore
                                        let mut iter = project_iter.clone();
                                        let mut file_iter = project_iter.clone();
                                        let mut folder_iter = project_iter.clone();

                                        if let Ok(file_path) = entry.path().as_path().strip_prefix(&project_dir.as_path()){
                                            let dir_name: [&dyn ToValue; 2] = [&file_path.to_str(), &file_path.to_str()];

                                            // debug!("entry path:'{:#?}'", entry.path());
                                            // if it's dir: 1.insert with folder_iter. 2.also add sub_dir files to treestore
                                            if entry.path().is_dir() {
                                                folder_iter = treestore.insert_with_values(Some(&folder_iter), None, &[0,1], &dir_name);
                                                debug!("folder_iter:'{:#?}'", treestore.get_string_from_iter(&folder_iter));

                                                // let sub_dir = entry.path().clone();
                                                // if let Ok(sub_entries) = fs::read_dir(entry.path()){
                                                //     for sub_entry in sub_entries {
                                                //         if let Ok(sub_dir_file) = sub_entry {
                                                //             if let Ok(file_path) = sub_dir_file.path().as_path().strip_prefix(&sub_dir.as_path()){
                                                //                 let dir_name: [&dyn ToValue; 2] = [&file_path.to_str(),&file_path.to_str()];
                                                //                 treestore.insert_with_values(Some(&iter), None, &[0,1], &dir_name);
                                                //             }
                                                //         }
                                                //     }
                                                // }
                                            } else {
                                                file_iter = treestore.insert_with_values(Some(&file_iter), None, &[0,1], &dir_name);
                                                debug!("file_iter:'{:#?}'", treestore.get_string_from_iter(&file_iter));
                                            }
                                        }
                                    }
                                }
                                // vec.append(&mut vec_files);
                                // debug!("treestore:'{:#?}'", treestore);
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
