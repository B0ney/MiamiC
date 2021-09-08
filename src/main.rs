use std::{env, io, fs};

mod save_converter;
use save_converter::{
    savefile::SaveFile,
    enums::SaveType,
};

fn main() {
    println!("--MiamiC-- \nSave converter for GTA: Vice City (PC versions)\nBy B0ney (https://github.com/B0ney)\n\nHow to use:\n   miamic 'YOUR_SAVE_PATH'\n");
    
    if let Err(e) = run_tool() {
        println!("{}", e);
    };

    println!("\nPress ENTER to continue.");
    drop(input());
}

fn run_tool() -> Result<(), String> {
    // Make sure the program can load the save file
    let save_path = get_save_path()?;
    let save_file = SaveFile::new(&save_path)?;

    println!("Loaded: {}\n", save_path);

    // Tell the user the verison of the save file and promt them if they wish to convert it
    let user_input: bool = prompt(&save_file.SaveType)?;

    if user_input {
        convert_save(save_file, &save_path)?
    } else {
        Err("User aborted".to_string())?
    };
    
    Ok(())
}

fn convert_save(mut save_file: SaveFile, save_path: &str) -> Result<(), String> {
    backup_save(&mut save_file, save_path)?;
    save_file.convert_save(save_path)?;
    
    Ok(()) 
}

fn backup_save(save_file: &mut SaveFile, save_path: &str) -> Result<(), String> {
    // Attempt to backup the original save flie before converting it. 
    // If this fails, ask the user if it's okay to make a backup using the save loaded in memory
    let backup_location = format!("{}.bak", save_path);

    if let Err(msg) = fs::copy(save_path, &backup_location) {
        println!("Cannot backup original save: {}\nMake backup using the loaded save? Y/n", msg);
        
        let user_choice = input()?;

        match user_choice.chars().nth(0) {
            Some('n') | Some('N') => Err("Conversion aborted.")?,

            _ => save_file.export(&backup_location),
        } 
    } else {
        println!("Original save backed up to: {}", &backup_location);
        Ok(())
    }
}

fn input() -> Result<String, String> {
    let mut user_input = String::new();

    io::stdin()
        .read_line(&mut user_input)
        .map_err(|e| e.to_string())?;

    Ok(user_input)
}

fn prompt(save_type: &SaveType) -> Result<bool, String> {
    match save_type {
        SaveType::Steam => {
            println!("STEAM version detected, convert to RETAIL? y/N? ");
        },
        SaveType::Retail => {
            println!("RETAIL version detected, convert to STEAM? y/N? ");
        },
        SaveType::Android | SaveType::IOS  => {
            Err("Android and IOS saves are not supported".to_string())?;
        }
        _ => {
            Err("Cannot determine save type".to_string())?;
        }
    };

    let user_choice = input()?;

    match user_choice.chars().nth(0) {
        Some('Y') | Some('y') => {
            println!("");
            Ok(true)
        }
        _ => {
            Ok(false)
        }
    }
}

fn get_save_path() -> Result<String, String> {
    match env::args().nth(1) {
        Some(file_path) => Ok(file_path.to_string()),
        None => Err("No save path provided".to_string())       
    }
}
