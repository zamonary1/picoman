pub mod apk_utils;
pub mod cli;
pub mod device_comm;
pub mod helpers;

use std::env::temp_dir;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::time::{Duration, SystemTime};
use std::{fs, path::PathBuf, sync::mpsc, thread};

use device_comm::DeviceName;
use eframe::egui::{self};
use egui::Color32;
use native_dialog::{DialogBuilder, MessageLevel};

fn main() {
    // println!("{:?}", temp_dir!().join("files_to_install"));
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        //first arg is the path to executable
        cli::parse_args();
        std::process::exit(0);
    }

    helpers::nix_workaround();

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let _ = fs::create_dir(temp_dir!());
    //check temp_dir!() in helpers.rs`

    let options = eframe::NativeOptions {
        // viewport: egui::ViewportBuilder::default().with_inner_size([240.0, 300.0]),
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([330.0, 300.0])
            .with_min_inner_size([330.0, 230.0]),
        ..Default::default()
    };

    match eframe::run_native(
        format!("Picoman {}", env!("CARGO_PKG_VERSION")).as_str(),
        options,
        Box::new(|_cc| Ok(Box::<PicomanApp>::default())),
    ) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("There was an error when starting GUI:\n{}", err);
            let _ = DialogBuilder::message()
                .set_level(MessageLevel::Error)
                .set_title("Error")
                .set_text(format!("There was an error when starting GUI:\n{}", err))
                .alert()
                .show();
            std::process::exit(1);
        }
    }
}

struct PicomanApp {
    is_processing: bool,
    is_dev_connected: bool,
    is_java_installed: bool,
    games_owned: bool,
    device: DeviceName,
    status: String,
    files: Vec<PathBuf>,
    last_dev_update_time: SystemTime,
    status_bool_receiver: Receiver<bool>,
    status_bool_sender: Sender<bool>,
    status_str_receiver: Receiver<String>,
    status_str_sender: Sender<String>,
}

impl Default for PicomanApp {
    fn default() -> Self {
        // let _app_label = format!("Picoman {}", env!("CARGO_PKG_VERSION"));
        let (status_tx, status_rx) = mpsc::channel::<bool>();
        let (status_str_tx, status_str_rx) = mpsc::channel::<String>();
        Self {
            is_processing: false,
            is_dev_connected: false,
            is_java_installed: apk_utils::Apktool::_is_command_installed(
                apk_utils::DownloadPath::apktool().java,
            ),
            games_owned: false,
            device: DeviceName::None,
            status: String::new(),
            files: vec![],
            // label: _app_label,
            last_dev_update_time: SystemTime::now(),
            status_bool_receiver: status_rx,
            status_bool_sender: status_tx,
            status_str_receiver: status_str_rx,
            status_str_sender: status_str_tx,
        }
    }
}

impl eframe::App for PicomanApp {
    //#[tokio::main]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.5);

            if self.last_dev_update_time.elapsed().unwrap() > Duration::from_secs(1) {
                //search for connected devices
                //update every second
                self.last_dev_update_time = SystemTime::now();
                self.device = device_comm::get_connected_device();

                match self.device {
                    //if device isn't none
                    DeviceName::None => self.is_dev_connected = false,
                    _ => self.is_dev_connected = true,
                }

                // self.is_dev_connected = device_comm::is_usb_connected(vendor_id, product_id);
            }

            // ui.heading(self.label.clone());

            // ui.separator();

            if self.is_dev_connected {
                ui.colored_label(Color32::GREEN, "Connected to device!");
            } else {
                ui.colored_label(Color32::RED, "Not connected to device.");
            }

            ui.separator();

            if !self.is_java_installed {
                ui.colored_label(Color32::RED, "Java is not installed!");
            }

            if ui.button("Add files").clicked() {
                let files_sel = DialogBuilder::file()
                    .add_filter("Android Package", &["apk"])
                    .open_multiple_file()
                    .show()
                    .unwrap_or(Vec::new());
                if files_sel.len() > 0 {
                    self.files.extend(files_sel);
                }
            }

            ui.horizontal(|ui| {
                ui.collapsing("Added files", |ui| {
                    let mut elem_to_delete = usize::MAX;
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for elem in 0..self.files.len() {
                            ui.horizontal(|ui| {
                                if ui.button("Delete").clicked() {
                                    println!("Deleted element {}!", elem);
                                    elem_to_delete = elem;
                                }
                                // Without ui.with_layout text doesn't cull and overflows from window edge
                                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.label(format!(
                                        "{}: {:?}",
                                        elem + 1,
                                        self.files[elem].file_name().unwrap()
                                    ));
                                }); //ui.with_layout
                            }); //ui.horizontal
                        }
                    }); //scroll are

                    if elem_to_delete != usize::MAX {
                        self.files.remove(elem_to_delete);
                    }
                    if self.files.len() == 0 {
                        ui.label("No files added!");
                    }
                }); //Added files tab
            });

            ui.label(format!("Total files: {}", self.files.len()));

            ui.checkbox(
                &mut self.games_owned,
                "I legally own or have permission to use these files",
            );

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        !self.is_processing
                            && self.is_java_installed
                            && self.is_dev_connected
                            && self.games_owned,
                        egui::Button::new("Patch and upload"),
                    )
                    .clicked()
                {
                    self.is_processing = true;
                    handle_upload(&self);
                };
                if ui
                    .add_enabled(
                        !self.is_processing && self.is_java_installed && self.games_owned,
                        egui::Button::new("Patch and save"),
                    )
                    .clicked()
                {
                    self.is_processing = true;
                    handle_sign(
                        &self, None, //let user choose where to save
                    );
                    // send_worker.send(&self.files).unwrap();
                };
            });

            match self.status_bool_receiver.try_recv() {
                Ok(t) => {
                    //got something
                    self.is_processing = false;

                    if t == true {
                        //remove files only on success
                        for _ in self.files.clone() {
                            self.files.pop(); // delete all entries
                        }
                    }
                    // else {
                    //error
                    // self.status =
                    //     String::from("Something went wrong, try to reconnect the device.");
                    // }
                }
                Err(_) => (),
            }
            match self.status_str_receiver.try_recv() {
                Ok(t) => {
                    //got something
                    self.status = t;
                }
                Err(_) => (),
            }

            ui.label(self.status.clone());

            ui.add_visible(
                self.is_processing,
                egui::Label::new("Installation in progress"),
            );
            ui.add_visible(
                self.is_processing,
                egui::Label::new("Check device's screen for instructions"),
            );

            ui.add_visible(self.is_processing, egui::Spinner::new());
        });

        // zctx.requested_repaint_last_pass() { // redraw every second
        ctx.request_repaint_after_secs(1.0);
    }
}

// This function gets list of files that are going to be installed
// and connected device's vendor and product id. Then it tries to
// patch every APK and then pushes them individually to device

fn handle_upload(app: &PicomanApp) {
    // Need to init all variables before starting new thread

    // Note: every var that is initialised with app.*.clone()
    // just grabs the value from main app

    let tx_ok = app.status_bool_sender.clone();
    let tx_status_ref = app.status_str_sender.clone();

    let install_dir = temp_dir!().join("files_to_install");

    let files_cloned = app.files.clone();

    let _ = fs::remove_dir_all(install_dir.clone()); //remove all previously processed files
    fs::create_dir(install_dir.clone()).unwrap();

    let vendor = app.device.info().vendor;
    let product = app.device.info().product;

    // thread 1 patches files and
    // moves them to separate dir
    thread::spawn(move || {
        for file in files_cloned {
            //WIP
            let _ = tx_status_ref.send(format!("Status: Patching {:?}", file.clone()));

            let file_name_string = String::from(file.file_name().unwrap().to_str().unwrap());
            // looks complicated but basically grabs filename and stores in String

            if file_name_string.contains("-patched.apk") {
                println!("{} is already patched, proceeding", file_name_string);
                fs::copy(file, &install_dir).unwrap();
            } else {
                println!("{} is unpatched", file_name_string);

                let patched: PathBuf;

                match apk_utils::Apktool::patch_apk(file.as_path()) {
                    // try to patch apk and catch errors
                    Ok(file) => patched = file,
                    Err(e) => {
                        let _ = tx_ok.send(false);
                        let _ = tx_status_ref.send(format!("Error patching file {file:?}\n{e}"));
                        std::process::exit(1);
                    }
                }

                fs::copy(
                    patched,
                    &install_dir.join(file.file_name().unwrap()), // copy to install dir under the same name
                )
                .unwrap();
            }
        }
        println!("Finished patching.");

        println!("Starting upload");
        let dir_scan = fs::read_dir(&install_dir).unwrap();
        for file in dir_scan {
            let _ = tx_status_ref.send(format!("Status: Uploading {:?}", file));
            // println!("{:?}", &file);
            let file_path = file.unwrap().path().clone();

            //pushes the apk to connected device
            match device_comm::push_apk_id(&file_path.as_path(), vendor, product) {
                Ok(_) => println!("Successfully pushed {}", file_path.display()),
                Err(e) => {
                    println!("Failed to push {}: {e}", file_path.display());
                    let _ =
                        tx_status_ref.send(format!("Failed to push {}: {e}", file_path.display()));
                    tx_ok.send(false).unwrap();
                    std::process::exit(1);
                }
            }
        }
        let _ = tx_ok.send(true);
        let _ = tx_status_ref.send(format!("Status: OK"));

        println!("Finished upload!");
    }); //thread end
}

fn handle_sign(app: &PicomanApp, out: Option<PathBuf>) {
    let (tx_fil, rx_fil) = mpsc::channel::<Vec<PathBuf>>();
    let (tx_out, rx_out) = mpsc::channel::<Option<PathBuf>>();

    let tx_ok = app.status_bool_sender.clone();
    let tx_status_ref = app.status_str_sender.clone();

    tx_fil.send(app.files.clone()).unwrap();
    //
    tx_out.send(out).unwrap();
    // This sends a message to thread containing the path to save a file

    if !apk_utils::Apktool::_is_command_installed("java") {
        println!("Java is not installed");
        let _ = tx_ok.send(false);
        let _ = tx_status_ref.send(String::from("Error: Java is not installed."));
    }

    thread::spawn(move || {
        let files = rx_fil.recv().unwrap();

        let out = rx_out.recv().unwrap();

        let mut out_mut = out.clone();
        // Receive the message and clone
        // the output dir to mutate it later

        let mut can_continue = true;

        let _ = tx_status_ref.send(String::from(
            "Status: Asking the directory to save the file",
        ));

        if out == None {
            let pick = DialogBuilder::file().open_single_dir().show().unwrap();
            // builder of directory selector gui
            if pick != None {
                out_mut = pick;
                // out_mut still contains PathBuf wrapped in Option<>
            } else {
                let _ = tx_ok.send(false);
                //indicate that something is wrong to main thread
                can_continue = false;
            }
        }

        if can_continue {
            for file in files {
                let _ = tx_status_ref.send(format!("Status: Patching file {:?}", file.clone()));

                // let patched = apk_utils::Apktool::patch_apk(file.as_path()).unwrap();
                let patched: PathBuf;
                match apk_utils::Apktool::patch_apk(file.as_path()) {
                    Ok(file) => patched = file,
                    Err(e) => {
                        let _ = tx_ok.send(false);
                        let _ = tx_status_ref.send(format!("Error patching file {file:?}\n{e}"));
                        std::process::exit(1);
                    }
                }
                let filename = file.file_stem().unwrap().to_str().unwrap();
                let out_named = &out_mut
                    .clone()
                    .unwrap()
                    .join(format!("{filename}-patched.apk"));
                // get filename of currently patching file, add -patched
                // to it and save in directory stored in out_mut
                println!("{:#?}", out_named);
                fs::copy(patched, out_named).unwrap();
            }
        }
        let _ = tx_ok.send(true); // everything was ok
        let _ = tx_status_ref.send(format!("Status: OK"));
    });
}
