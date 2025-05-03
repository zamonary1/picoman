pub(crate) use clap::{arg, command, value_parser, Command};
// use colored::Colorize;

// use std::fs;
use std::path::PathBuf;

use crate::apk_utils::Apktool;
use crate::device_comm::{self, push_apk_id, DeviceName};

#[macro_export]
macro_rules! extension_str {
    ($path:expr) => {
        $path.extension().unwrap().to_str().unwrap()
    };
}

pub fn parse_args() {
    let matches = command!() // requires `cargo` feature
        .subcommand(
            Command::new("devices")
                .about("ADB device operations (debug)")
                .arg(arg!(
                    -l --list "Check connected devices"
                ))
        )

        .subcommand(
            Command::new("apk")
                .about("APK operations")
                .arg(
                    arg!(
                    --unwrap <FILE> "Disassembles APK file (debug)" )
                    .required(false)
                    .value_parser(value_parser!(PathBuf)),

                )

                .arg(
                    arg!(
                    -o --out <PATH> "Where to put processed files" )
                    .required(false)
                    .default_value("picoman-out")
                    .value_parser(value_parser!(PathBuf)),

                )

                .arg(
                    arg!(
                    --manifest <FILE> "Patches disassembled APK's AndroidManifest.xml (debug)" )
                    .required(false)
                    .value_parser(value_parser!(PathBuf)),

                )

                .arg(
                    arg!(
                    --patch <FILE> "Patches APK" )
                    .required(false)
                    .value_parser(value_parser!(PathBuf)),

                )

                .arg(
                    arg!(
                    -i --install <FILE> "Patches and installs APK on your device, connected via USB" )
                    .required(false)
                    .value_parser(value_parser!(PathBuf)),

                )
        )

        .get_matches();

    if let Some(matches) = matches.subcommand_matches("devices") {
        if matches.get_flag("list") {
            match device_comm::get_connected_device() {
                DeviceName::None => println!("No devices detected."),
                DeviceName::Pico4Neo3 => println!("Pico 4 or Pico neo 3 is detected."),
                _ => (),
            }
            // println!("{} device(s) connected", device_comm::devices_connected());
        }
    }

    if let Some(matches) = matches.subcommand_matches("apk") {
        // if let Some(out) = matches.get_one::<PathBuf>("out"){
        //     let out_path = &out;
        // }

        if let Some(unwrap) = matches.get_one::<PathBuf>("unwrap") {
            println!("unwrap {:?}", &unwrap);
            Apktool::decomp_apk(
                &unwrap,
                matches.get_one::<PathBuf>("out").expect("required"),
            );
        }

        if let Some(unwrap) = matches.get_one::<PathBuf>("patch") {
            Apktool::patch_apk(&unwrap).expect("Error occured");
        }

        if let Some(unwrap) = matches.get_one::<PathBuf>("manifest") {
            Apktool::patch_manifest(&unwrap);
        }

        if let Some(unwrap) = matches.get_one::<PathBuf>("install") {
            let patched_apk = Apktool::patch_apk(&unwrap).expect("Error occured");
            // device_comm::push_apk(&patched_apk.to_path_buf());
            let device = device_comm::get_connected_device();
            let vendor_id = device.info().vendor;
            let product_id = device.info().product;
            match device {
                DeviceName::None => println!("No devices detected."),
                DeviceName::Pico4Neo3 => println!("Pico 4 or Pico neo 3 is detected."),
                _ => println!("Unknown device is detected."),
            }
            push_apk_id(&patched_apk.to_path_buf(), vendor_id, product_id).unwrap();
        }
    }

    // if let Some(matches) = matches.subcommand_matches("install") {
    //     match device_comm::get_connected_device() {

    //         device_comm::DeviceName::None => panic!("No active devices connected via ADB!"),
    //         _ => (), //if exists, just continue
    //     }

    //     let apk_path_buf = matches.get_one::<PathBuf>("file").unwrap();
    //     //let apk_path = apk_pathBuf.as_path();

    //     if !fs::exists(apk_path_buf).unwrap() {
    //         panic!("File does not exist!");
    //     }

    //     let mut files: Vec<PathBuf> = vec![];
    //     let extensions = ["apk"]; //todo: support zip files

    //     if apk_path_buf.is_dir() {
    //         //get list of all installable files
    //         for entry in fs::read_dir(apk_path_buf).unwrap() {
    //             let entry = entry.unwrap();
    //             let path = entry.path();
    //             if !path.extension().is_none() {
    //                 if extensions.contains(&extension_str!(&path)) {
    //                     files.push(path);
    //                 }
    //             }
    //         }
    //     }
    //     if apk_path_buf.is_file() {
    //         if extensions.contains(&extension_str!(&apk_path_buf)) {
    //             files.push(apk_path_buf.clone());
    //         }
    //     }

    //     for file in files {
    //         println!("Install: {:?}", file.as_path());

    //         let file_out = Apktool::patch_apk(&file.as_path()).unwrap();
    //         device_comm::push_apk(file_out.as_path());
    //     }

    //     println!(
    //         "{}",
    //         Colorize::green("All files installed!\nThank you for using picoman!")
    //     );
    // }
}
