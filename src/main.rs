use std::{env, io, fs};

mod save_converter;
use save_converter::{
    savefile::SaveFile,
    enums::SaveType,
};

fn main() {
    println!("--MiamiC-- \nBy B0ney (https://github.com/B0ney)\nSave converter for GTA: Vice City (PC versions)\n\nHow to use:\n   miamic 'YOUR_SAVE_PATH'\n");
    
    if let Err(e) = run_tool() {
        println!("{}", e)
    };
}


fn run_tool() -> Result<(), String> {
    // Make sure the program can load the save file
    let save_path = get_save_path()?;
    let save_file = SaveFile::new(&save_path)?;

    println!("Successfully Loaded: {}", save_path);

    // Tell the user the verison of the save file and promt them if they wish to convert it
    let save_type = save_file.SaveType.clone();
    let user_input: bool = prompt(save_type)?;

    if user_input {
        convert_save(save_file, &save_path)?;
    } else {
        Err("User aborted".to_string())?
    };

    Ok(())
}

fn convert_save(mut save_file: SaveFile, save_path: &str) -> Result<(), String> {
    // Attempt to make a backup of the original save file before converting it
    let backup_location = format!("{}.bak", save_path);

    fs::copy(save_path, &backup_location).map_err(|e| e.to_string())?;

    println!("Original save backed up to: {}", &backup_location);

    save_file.convert_save(save_path)?;

    Ok(()) 
}

fn input() -> Result<String, String> {
    let mut user_input = String::new();

    io::stdin()
        .read_line(&mut user_input)
        .map_err(|e| e.to_string())?;

    Ok(user_input)
}

fn prompt(save_type: SaveType) -> Result<bool, String> {
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
            Err("Cannot determine Save".to_string())?;
        }
    }

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
