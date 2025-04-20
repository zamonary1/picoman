use adb_client::ADBDeviceExt;
use adb_client::ADBServer;
use adb_client::ADBUSBDevice;

use eframe::Result;
use rusb::{Context, UsbContext};

// use core::error;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::Path;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter)]
pub enum DeviceName {
    Pico4Neo3,
    None,
}
pub struct DeviceStruct {
    pub name: DeviceName,
    pub product: u16,
    pub vendor: u16,
}

pub fn device_info(n: &DeviceName) -> DeviceStruct {
    match n {
        DeviceName::Pico4Neo3 => DeviceStruct {
            name: DeviceName::Pico4Neo3,
            product: 0x00b7,
            vendor: 0x2d40,
        },
        DeviceName::None => DeviceStruct {
            name: DeviceName::None,
            product: 0x0,
            vendor: 0x0,
        },
    }
}

fn adb_server() -> ADBServer {
    let server_ip = Ipv4Addr::new(127, 0, 0, 1);
    let server_port = 5037;
    let server = ADBServer::new(SocketAddrV4::new(server_ip, server_port));
    return server;
}

pub fn is_usb_connected(vendor_id: u16, product_id: u16) -> bool {
    // simply checks if device with provided vendor and
    // product IDs is connected

    let context = Context::new().unwrap();
    let devices = context.devices().unwrap();
    // iterate through connected devices and check if one
    // of then is the one we're looking for

    // p.s. may be improved
    for device in devices.iter() {
        let device_desc = device.device_descriptor().unwrap();

        if device_desc.vendor_id() == vendor_id && device_desc.product_id() == product_id {
            return true;
        }
    }

    false
}

pub fn get_connected_device() -> DeviceName {
    // iterates through every device in device list
    // and returns the first one found
    for dev in DeviceName::iter() {
        let vendor = device_info(&dev).vendor;
        let product = device_info(&dev).product;
        // print!("{:?} is ", dev);
        if is_usb_connected(vendor, product) {
            // println!("connected!\n");
            return dev;
        } else {
            // println!("not connected.");
        }
    }
    // print!("\n");
    DeviceName::None
}

pub fn push_apk_first(apk_file: &Path) -> Result<(), adb_client::RustADBError> {
    // installs apk to any connected device
    return adb_server().get_device().unwrap().install(apk_file);
}

pub fn push_apk_id(
    apk_file: &Path,
    vendor_id: u16,
    product_id: u16,
) -> Result<(), adb_client::RustADBError> {
    // installs apk using device and vendor id
    //
    // if device is found, tries to install
    // and returns the status
    match ADBUSBDevice::new(vendor_id, product_id) {
        Ok(mut device) => return device.install(&apk_file),
        Err(e) => return Err(e),
    }
}
