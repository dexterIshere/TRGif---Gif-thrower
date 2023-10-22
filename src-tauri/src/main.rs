// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, thread};

use arboard::Clipboard;
use device_query::{DeviceEvents, DeviceQuery, DeviceState, Keycode};
use rand::Rng;
use rdev::{simulate, EventType, Key};
use serde_derive::{Deserialize, Serialize};
use tauri::api::path::{app_local_data_dir, config_dir};
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use toml::map::Map;
use toml::Value;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug)]
struct GifList {
    giflist: Vec<Link>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Link {
    link: String,
}

fn get_local_data_path() -> Result<PathBuf, String> {
    let binding = tauri::generate_context!();
    let config = binding.config();
    let local_data_path = app_local_data_dir(&config)
        .ok_or_else(|| "Failed to get local data directory".to_string())?;
    let emotions_dir = local_data_path.join("emotions");
    Ok(emotions_dir)
}

#[tauri::command]
fn get_n_read(emotion: &str) -> Result<(PathBuf, String), String> {
    let emotions_dir = get_local_data_path()?;
    let file_path = emotions_dir.join(format!("{}.json", emotion));
    fs::create_dir_all(&emotions_dir).map_err(|e| e.to_string())?;

    OpenOptions::new()
        .write(true)
        .create(true)
        .open(&file_path)
        .map_err(|e| e.to_string())?;

    let contenu = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    Ok((file_path, contenu))
}

#[tauri::command]
fn add_to_list(emotion: &str, valeur: &str) -> Result<(), String> {
    let (file_path, content) = get_n_read(emotion)?;
    let mut gif_list: GifList =
        serde_json::from_str(&content).unwrap_or(GifList { giflist: vec![] });
    gif_list.giflist.push(Link {
        link: valeur.into(),
    });

    let update_content = serde_json::to_string_pretty(&gif_list).map_err(|err| err.to_string())?;

    fs::write(&file_path, &update_content).map_err(|err| err.to_string())?;

    Ok(())
}

#[tauri::command]
fn random_gif(emotion: &str) -> Result<String, String> {
    let (_file_path, content) = get_n_read(emotion)?;
    let data: GifList = serde_json::from_str(&content).map_err(|err| err.to_string())?;

    let mut rng = rand::thread_rng();
    let rand_index = rng.gen_range(0..data.giflist.len());
    let return_random = format!("{}", data.giflist[rand_index].link.clone());
    Ok(return_random)
}

#[tauri::command]
fn new_keys(emotion: &str) -> Result<String, String> {
    let device_state = DeviceState::new();
    let mut key_pressed: Vec<Keycode> = Vec::new();

    loop {
        let keys = device_state.get_keys();
        if !keys.is_empty() {
            for key in keys.iter() {
                if !key_pressed.contains(key) {
                    key_pressed.push(*key);
                }
            }
        } else {
            if !key_pressed.is_empty() {
                break;
            }
        }
    }

    let mut new_cmd = String::new();

    for (i, key) in key_pressed.iter().enumerate() {
        if i > 0 {
            new_cmd.push_str(" + ");
        }
        new_cmd.push_str(&format!("{:?}", key));
    }

    insert_key(&emotion, &new_cmd)?;

    Ok(new_cmd)
}

#[tauri::command]
fn new_emo(name: &str) -> Result<(), String> {
    let _ = get_n_read(name);
    Ok(())
}

fn main() {
    //config
    let _ = watch_emo_folder();

    //tray
    let leave_item = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(leave_item)
        .add_native_item(SystemTrayMenuItem::Separator);

    // keys
    listen_keys();

    tauri::Builder::default()
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => app.restart(),
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![add_to_list, new_keys, new_emo])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}

fn copy_pasta(emotion: &str) {
    let rand_gif_str = random_gif(emotion).unwrap();
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(rand_gif_str.to_owned()).unwrap();

    simulate(&EventType::KeyPress(Key::ControlLeft)).unwrap();
    simulate(&EventType::KeyPress(Key::KeyV)).unwrap();
    thread::sleep(Duration::from_millis(100));
    simulate(&EventType::KeyRelease(Key::KeyV)).unwrap();
    simulate(&EventType::KeyRelease(Key::ControlLeft)).unwrap();
    thread::sleep(Duration::from_millis(100));

    simulate(&EventType::KeyPress(Key::Return)).unwrap();
    simulate(&EventType::KeyRelease(Key::Return)).unwrap();
}

fn listen_keys() {
    let device_state = DeviceState::new();
    thread::spawn(move || {
        let _guard = device_state.on_key_down(|key| match key {
            Keycode::F2 => {
                let emotion = "fun";
                copy_pasta(&emotion)
            }
            Keycode::F3 => {
                let emotion = "cringe";
                copy_pasta(&emotion)
            }
            Keycode::F4 => {
                let emotion = "choked-boar";
                copy_pasta(&emotion)
            }
            _ => (),
        });
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

fn watch_emo_folder() -> Result<Vec<String>, String> {
    let emotion_dir = get_local_data_path()?;
    let mut emo_box = Vec::new();

    for entry in WalkDir::new(emotion_dir) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            if let Some(emo_name) = entry.file_name().to_str() {
                emo_box.push(emo_name.to_string());
            }
        }
    }

    println!("{:#?}", emo_box);

    Ok(emo_box)
}

fn fetch_keyset() -> Result<PathBuf, String> {
    let config_path = config_dir().ok_or_else(|| "Failed to get config directory".to_string())?;
    let trgif_roaming_dir = config_path.join(format!("TRGif"));
    if !trgif_roaming_dir.exists() {
        create_dir_all(&trgif_roaming_dir).map_err(|e| e.to_string())?;
    }
    let ini_file = trgif_roaming_dir.join(format!("config.toml"));

    if !ini_file.exists() {
        let mut default_section = toml::value::Table::new();
        default_section.insert(
            "settings".to_string(),
            Value::Table(toml::value::Table::new()),
        );

        let toml_str =
            toml::to_string(&Value::Table(default_section)).map_err(|e| e.to_string())?;

        OpenOptions::new()
            .write(true)
            .create(true)
            .open(&ini_file)
            .and_then(|mut file| file.write_all(toml_str.as_bytes()))
            .map_err(|e| e.to_string())?;
    }
    println!("{:?}", ini_file);

    Ok(ini_file)
}

fn read_ini() -> Result<String, String> {
    let ini_file = fetch_keyset()?;
    let mut file_content = String::new();

    File::open(ini_file)
        .expect("Fichier non trouvé!")
        .read_to_string(&mut file_content)
        .expect("Erreur lors de la lecture du fichier");

    Ok(file_content)
}

fn save_config(config: &Map<String, Value>) -> Result<(), String> {
    let path = fetch_keyset()?;
    let toml_str = toml::to_string(&config).unwrap();

    File::create(path)
        .unwrap()
        .write_all(toml_str.as_bytes())
        .unwrap();
    Ok(())
}

fn insert_key(emotion: &str, value: &str) -> Result<(), String> {
    let file_content = read_ini()?;
    let mut config_add: toml::value::Table = toml::from_str(&file_content).unwrap();
    let key = emotion;
    let value = value;

    if let Some(db_map) = config_add
        .get_mut("settings")
        .and_then(|v| v.as_table_mut())
    {
        db_map.insert(key.to_string(), Value::String(value.to_string()));
    }

    save_config(&config_add)
}

fn rmv_key(emotion: &str) -> Result<(), String> {
    let file_content = read_ini()?;
    let mut config_rmv: toml::value::Table = toml::from_str(&file_content).unwrap();
    let key = emotion;

    if let Some(db_map) = config_rmv
        .get_mut("settings")
        .and_then(|v| v.as_table_mut())
    {
        db_map.remove(key);
    }

    save_config(&config_rmv)
}
