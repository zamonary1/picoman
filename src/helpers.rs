use colored::Colorize;
use sysinfo::System;

#[macro_export]
macro_rules! temp_dir {
    () => {
        std::env::temp_dir().join("picoman")
    };
}

pub fn nix_workaround() {
    if System::name().unwrap() == "NixOS" {
        println!(
            "{}{}",
            "Running on NixOS
If you can't launch the gui, run:"
                .red(),
            "
nix-shell --run \"just runnix\""
                .green()
        );
    }
}
