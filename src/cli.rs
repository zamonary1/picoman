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
                    .default_value("picoman_files")
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
                DeviceName::None => error!("No devices detected."),
                DeviceName::Pico4Neo3 => info!("Pico 4 or Pico neo 3 is detected."),
                _ => (),
            }
        }
    }

    if let Some(matches) = matches.subcommand_matches("apk") {
        // if let Some(out) = matches.get_one::<PathBuf>("out"){
        //     let out_path = &out;
        // }

        if let Some(unwrap) = matches.get_one::<PathBuf>("unwrap") {
            info!("unwrap {:?}", &unwrap);
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
                DeviceName::None => error!("No devices detected."),
                DeviceName::Pico4Neo3 => info!("Pico 4 or Pico neo 3 is detected."),
                _ => (),
            }
            push_apk_id(&patched_apk.to_path_buf(), vendor_id, product_id).unwrap();
        }
    }
}
