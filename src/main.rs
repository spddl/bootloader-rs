use dialoguer::{theme::ColorfulTheme, Select};
use dialoguer::console;

// use console::{style, Style, StyledObject, Term};
use std::process::Command;
use system_shutdown::force_reboot;

use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut target_bootname = String::from("");
    if args.len() > 1 {
        target_bootname = args[1..args.len()].join(" ");
    }

    let (display_order, default_guid) = get_bcd_bootloader();
    let mut default_index: usize = 0;
    let guid_list: Vec<&str> = display_order.split("\n").collect();

    let mut theme = ColorfulTheme::default();
    theme.active_item_prefix = console::style(">".to_string()).for_stderr().green();

    let mut name_list: Vec<String> = vec![];
    for (index, guid) in guid_list.clone().iter().enumerate() {
        let bootname = get_bcd_entry(guid);

        if default_guid == *guid.clone() {
            default_index = index;
        }

        if target_bootname != "" && target_bootname.to_lowercase() == bootname.clone().to_lowercase() {
            if *guid != default_guid {
                set_bcd_default(guid)
            }
            match force_reboot() {
                Ok(_) => println!("Rebooting ..."),
                Err(error) => eprintln!("Failed to reboot: {}", error),
            }
        }

        name_list.push(bootname);
    }

    if let Ok(select) = Select::with_theme(&theme)
        .items(&name_list)
        .default(default_index)
        .interact()
    {
        if guid_list[select] != default_guid {
            set_bcd_default(guid_list[select])
        }
    };

    if let Ok(select) = Select::with_theme(&theme)
        .items(&vec![
            "Reboot",
            "No Reboot",
        ])
        .default(0)
        .interact()
    {
        if select == 0 {
            // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-exitwindowsex
            match force_reboot() {
                Ok(_) => println!("Rebooting ..."),
                Err(error) => eprintln!("Failed to reboot: {}", error),
            }
        }
    };
}

fn set_bcd_default(guid: &str) {
    // https://docs.microsoft.com/en-us/windows-hardware/drivers/devtest/changing-the-default-boot-entry
    Command::new("bcdedit")
        .arg("/default")
        .arg(guid)
        .output()
        .unwrap();
}

fn get_bcd_entry(guid: &str) -> String {
    let bcde_library_type_description = String::from("12000004");
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    return match hklm.open_subkey(format!("BCD00000000\\Objects\\{}\\Elements\\{}", guid, bcde_library_type_description)) {
        Ok(type_display_order) => type_display_order.get_value("Element").unwrap(),
        Err(error) => panic!("ERR: bcde_bootmgr_type_display_order {:?}", error),
    };
}

fn get_bcd_bootloader() -> (String, String) {
    let guid_windows_bootmgr = String::from("{9DEA862C-5CDD-4E70-ACC1-F32B344D4795}");
    let bcde_bootmgr_type_display_order = String::from("24000001");
    let bcde_bootmgr_type_default_object = String::from("23000003");

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let display_order = match hklm.open_subkey(format!("BCD00000000\\Objects\\{}\\Elements\\{}", guid_windows_bootmgr, bcde_bootmgr_type_display_order)) {
        Ok(type_display_order) => type_display_order.get_value("Element").unwrap(),
        Err(error) => panic!("ERR: bcde_bootmgr_type_display_order {:?}", error),
    };

    let default_guid = match hklm.open_subkey(format!("BCD00000000\\Objects\\{}\\Elements\\{}", guid_windows_bootmgr, bcde_bootmgr_type_default_object)) {
        Ok(type_default_object) => type_default_object.get_value("Element").unwrap(),
        Err(error) => panic!("ERR: bcde_bootmgr_type_default_object {:?}", error),
    };

    return (display_order, default_guid);
}
