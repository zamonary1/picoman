use std::fmt::Error;
use std::path::Path;
use std::str::from_utf8;

use std::fs::File;
use std::io::prelude::*;

use colored::Colorize;
use rand::seq::IndexedRandom;
// use random_string::generate;

use edit_xml::Document;

use sha256::try_digest;
use std::env;

use std::process::Command;

use crate::temp_dir;

pub struct DownloadPath {
    url: &'static str,
    filename: &'static str,
    sha256: &'static str,
    pub java: &'static str,
}

impl DownloadPath {
    pub fn apktool() -> DownloadPath {
        DownloadPath {
            url: "https://bitbucket.org/iBotPeaches/apktool/downloads/apktool_2.11.0.jar",
            filename: "picoman_files/apktool.jar",
            sha256: "8fdc17c6fe2e6d80d71b8718eb2a5d0379f1cc7139ae777f6a499ce397b26f54",
            #[cfg(unix)]
            java: "java",
            #[cfg(target_os = "windows")]
            java: "java.exe",
        }
    }
    pub fn uber_apk_signer() -> DownloadPath {
        DownloadPath {
            url: "https://github.com/patrickfav/uber-apk-signer/releases/download/v1.3.0/uber-apk-signer-1.3.0.jar",
            filename: "picoman_files/uber_apk_signer.jar",
            sha256: "e1299fd6fcf4da527dd53735b56127e8ea922a321128123b9c32d619bba1d835",
            #[cfg(unix)]
            java: "java",
            #[cfg(target_os = "windows")]
            java: "java.exe",
        }
    }
}

pub struct Apktool {}

impl Apktool {
    fn _download_file(delete_old: bool, url: &DownloadPath) {
        if delete_old {
            let _ = std::fs::remove_file(url.filename);
        }

        let mut resp = reqwest::blocking::get(url.url).expect("request failed");

        let mut out = File::create(url.filename).expect("failed to create file");

        std::io::copy(&mut resp, &mut out).expect("failed to copy content");
    }

    pub fn download_and_check_file(url: DownloadPath) -> Result<(), Box<dyn std::error::Error>> {
        //This function downloads and checks the file integrity of the given file.

        // check is file present
        if !Path::new(&url.filename).exists() {
            info!(
                "{} is not found, downloading from {} ...",
                &url.filename, &url.url
            );
            Self::_download_file(false, &url);
            info!("{} downloaded succesfully!", &url.filename);
        }

        // check file hash
        let file = Path::new(&url.filename);
        let hash = try_digest(&file).unwrap();

        if hash == url.sha256 {
            info!("{} hash verified, proceeding", &url.filename);
            return Ok(());
        } else {
            info!(
                "{} is corrupted! Downloading from {} ...",
                &url.filename, &url.url
            );
            Self::_download_file(true, &url);

            let hash_new = try_digest(&file).unwrap();

            if hash_new == url.sha256 {
                info!("{} downloaded and verified, proceeding", &url.filename);
                return Ok(());
            } else {
                return Err(From::from(format!("Apktool hash doesn't match!")));
            }
        }
    }

    pub fn _is_command_installed(command: &str) -> bool {
        //linux and macos
        let mut delimeter = ':';

        //in windows PATH envvar is sepatared by semicolon
        if cfg!(target_os = "windows") {
            delimeter = ';';
        }

        match env::var("PATH") {
            Ok(path) => {
                let path_dirs: Vec<_> = path.split(delimeter).collect();
                for dir in path_dirs {
                    let exe_path = Path::new(dir).join(command);
                    if exe_path.exists() {
                        return true;
                    }
                }
                false
            }
            Err(_) => {
                error!("Failed to retrieve PATH environment variable.");
                false
            }
        }
    }

    pub fn decomp_apk(path: &Path, out: &Path) -> Option<Error> {
        let java = DownloadPath::apktool().java;
        if Self::_is_command_installed(java) {
            let _ = Self::download_and_check_file(DownloadPath::apktool());
            let output = Command::new(java)
                .arg("-jar")
                .arg(DownloadPath::apktool().filename)
                .arg("d") //decompile
                // .arg("-i")
                .arg(path)
                .arg("-o")
                .arg(out)
                .output()
                .expect("Java process failed to start. Is java installed correctly?");

            info!(
                "Invoking apktool decompiler\nStdout: {}\nStderr: {}",
                from_utf8(output.stdout.as_slice()).unwrap_or_default(),
                from_utf8(output.stderr.as_slice()).unwrap_or_default()
            );

            None
        } else {
            return Some(Error);
        }
    }

    pub fn comp_apk(path: &Path, out: &Path) -> Option<Error> {
        let java = DownloadPath::apktool().java;
        if Self::_is_command_installed(java) {
            let _ = Self::download_and_check_file(DownloadPath::apktool());
            let output = Command::new(java)
                .arg("-jar")
                .arg(DownloadPath::apktool().filename)
                .arg("b") //build
                // .arg("-i")
                .arg(path)
                .arg("-o")
                .arg(out)
                .output()
                .expect("Java process failed to start. Is java installed correctly?");

            info!(
                "Invoking apktool builder\nStdout: {}\nStderr: {}",
                from_utf8(output.stdout.as_slice()).unwrap_or_default(),
                from_utf8(output.stderr.as_slice()).unwrap_or_default()
            );

            None
        } else {
            return Some(Error);
        }
    }

    pub fn patch_manifest(path: &Path) -> Option<Error> {
        //, apk_info: ApkInfo) {
        // let charset = "abcdefghijklmnopqrstuvwxyz1234567890";

        const WORDS1: [&str; 9] = [
            "funny",
            "spectacular",
            "amazing",
            "smelly",
            "delicious",
            "big",
            "carnivore",
            "mighty",
            "gentle",
        ];
        const WORDS2: [&str; 10] = [
            "apple",
            "banana",
            "build",
            "game",
            "program",
            "code",
            "object",
            "something",
            "click",
            "thing",
        ];

        if let Err(_) = std::fs::copy(&path, &path.with_file_name("Manifest-old.xml")) {
            return Some(Error);
        }

        let xml_contents = //Document::parse_file() is broken
            std::fs::read_to_string(path)
            .unwrap_or(String::new());

        if xml_contents == String::new() {
            return Some(Error);
        }

        let mut doc = Document::parse_str(xml_contents.as_str())
            .expect(format!("Unable to parse xml file {:?}", path).as_str());
        let doc_immut = Document::parse_file(&path)
            .expect(format!("Unable to parse xml file {:?}", path).as_str());

        let doc_root = doc.root_element().unwrap();

        info!(
            "XML contents:\n{} \nRoot element: \n************\n{:?}",
            xml_contents, doc_root
        );

        info!(
            "Document is root: {}, {:?}",
            doc_root.is_root(&doc_immut),
            doc_root.attribute(&doc_immut, "package")
        );

        info!(
            "Root's children:\n{:?}\n{:?}\n{:?}",
            doc_root.child_elements(&doc_immut),
            doc_root.child_elements(&doc_immut)[1].name(&doc_immut),
            doc_root.find(&doc_immut, "application")
        );

        //let manifest_xml = doc_root.clone();
        // let manifest_xml = doc.store;

        let application_xml = doc_root.find(&doc, "application");

        let package_xml = doc_root.attribute(&doc_immut, "package");
        let app_name_xml = application_xml
            .unwrap()
            .attribute(&doc_immut, "android:label");

        //.attribute(&doc, "label").unwrap();

        let mut app_name_new: String = String::from(app_name_xml.unwrap());
        app_name_new.push('+'); //add to the end of the app name

        let package_new: String = format!(
            "{}_{}_{}",
            package_xml.unwrap(),
            WORDS1.choose(&mut rand::rng()).unwrap(),
            WORDS2.choose(&mut rand::rng()).unwrap()
        );
        info!("New package name: {}", package_new);

        doc_root.set_attribute(&mut doc, "package", package_new);
        info!("Succesfully set the package name");

        let app_name_first_char = app_name_xml.unwrap().chars().next().unwrap();

        if app_name_first_char == '@' {
            error!("Name of the app is likely not stored in AndroidManifest.xml, skipping");
        } else {
            let new_app_name = format!(
                "{}{}",
                application_xml.unwrap().attribute(&doc, "label").unwrap(),
                app_name_new
            );
            application_xml
                .unwrap()
                .set_attribute(&mut doc, "label", new_app_name);
        }

        let new_xml = doc.write_str().unwrap();

        write!(File::create(path).unwrap(), "{}", new_xml).unwrap();

        None
    }

    // WIP
    // Sign file with v2 android signature using externl tool
    fn sign_apk(path: &Path) -> Option<Error> {
        let java = DownloadPath::apktool().java;
        if Self::_is_command_installed(java) {
            let _ = Self::download_and_check_file(DownloadPath::uber_apk_signer());
            let output = Command::new(java)
                .arg("-jar")
                .arg(DownloadPath::uber_apk_signer().filename)
                .arg("--overwrite")
                .arg("-apks")
                .arg(path)
                .output() // capture output
                .expect("Java process failed to start. Is java installed correctly?");

            info!(
                "Invoking Uber Apk Signer\nStdout: {}\nStderr: {}",
                from_utf8(output.stdout.as_slice()).unwrap_or_default(),
                from_utf8(output.stderr.as_slice()).unwrap_or_default()
            );

            return None;
        } else {
            return Some(Error);
        }
    }

    //todo: return errors
    pub fn patch_apk(
        file_path: &Path,
        //package_name: &str,
        //app_name_add: &str,
        //overwrite: bool,
    ) -> Result<std::path::PathBuf, Error> {
        let mut exec_dir = std::env::current_exe().unwrap();
        exec_dir.pop();

        let mut temp_dir = temp_dir!();
        temp_dir.push("picoman_temp");

        let apk_in = temp_dir.clone().join("in.apk");
        let apk_patched = exec_dir.clone().join("patched.apk");

        info!("Cleaning old files...");

        match std::fs::remove_dir_all(temp_dir.clone()) {
            Ok(_t) => info!("Removed files in {:?}", temp_dir.clone()),
            Err(err) => error!("Error while deleting directory: {}", err),
        }

        info!(
            "Copying {:?} to {:?}...",
            file_path.file_name(),
            temp_dir.clone()
        );
        std::fs::create_dir(temp_dir.clone()).unwrap();
        let result = std::fs::copy(file_path, apk_in.clone());
        info!("{:?}", result);

        info!("Disassembling file...");
        let status = Self::decomp_apk(apk_in.as_path(), temp_dir.clone().join("decomp").as_path());
        match status {
            Some(err) => return Err(err),
            _ => (),
        }

        info!("Patching metadata...");
        let status = Self::patch_manifest(
            temp_dir
                .clone()
                .join("decomp")
                .join("AndroidManifest.xml")
                .as_path(),
        );
        match status {
            Some(err) => return Err(err),
            _ => (),
        }

        //      Remove old signatures, if present
        //vvvvv - ignore errors
        let _ = std::fs::remove_file(
            temp_dir
                .clone()
                .join("decomp")
                .join("META-INF")
                .join("SIGNKEY.RSA"),
        );
        let _ = std::fs::remove_file(
            temp_dir
                .clone()
                .join("decomp")
                .join("META-INF")
                .join("SIGNKEY.SF"),
        );

        info!("Assembling apk...");
        let status = Self::comp_apk(
            temp_dir.clone().join("decomp").as_path(),
            temp_dir.clone().join("out.apk").as_path(),
        );
        match status {
            Some(err) => return Err(err),
            _ => (),
        }

        info!("Signing apk...");
        let status = Self::sign_apk(temp_dir.clone().join("out.apk").as_path());
        match status {
            Some(err) => return Err(err),
            _ => (),
        }

        std::fs::copy(
            temp_dir.clone().join("out.apk").as_path(),
            apk_patched.clone(),
        )
        .unwrap();

        match std::fs::remove_dir_all(temp_dir.clone()) {
            Ok(_t) => info!("Removed files in {:?}", temp_dir.clone()),
            Err(err) => error!("Error while deleting directory: {}", err),
        }

        info!(
            "{} {}",
            Colorize::green("Succesfully patched file! It's saved as"),
            Colorize::green(apk_patched.clone().to_str().unwrap()),
        );

        return Ok(apk_patched.clone());
    }
}
