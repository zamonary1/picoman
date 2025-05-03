use adb_client::ADBDeviceExt;
use adb_client::ADBUSBDevice;

use eframe::Result;
use rusb::{Context, UsbContext};

// use core::error;
use std::path::Path;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter)]
pub enum DeviceName {
    Pico4Neo3,
    TestPhone,
    None,
}
pub struct DeviceStruct {
    pub name: DeviceName,
    pub product: u16,
    pub vendor: u16,
}

impl DeviceName {
    pub fn info(&self) -> DeviceStruct {
        match self {
            DeviceName::Pico4Neo3 => DeviceStruct {
                name: DeviceName::Pico4Neo3,
                product: 0x00b7,
                vendor: 0x2d40,
            },
            DeviceName::TestPhone => DeviceStruct {
                name: DeviceName::TestPhone,
                product: 0x201c,
                vendor: 0x0e8d,
            },
            DeviceName::None => DeviceStruct {
                name: DeviceName::None,
                product: 0x0,
                vendor: 0x0,
            },
        }
    }
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
        let vendor = dev.info().vendor;
        let product = dev.info().product;
        if is_usb_connected(vendor, product) {
            return dev;
        } else {
        }
    }
    // print!("\n");
    DeviceName::None
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
    //

    // let mut dev1 = adb_client::ADBServerDevice;
    let mut device = ADBUSBDevice::new(vendor_id, product_id)?;

    // let mut n;
    // device.shell_command(&["df", "-h"], &mut n);
    device.install(&apk_file)?;

    // drop(device);

    return Ok(());
}
