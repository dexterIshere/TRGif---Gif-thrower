// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod shared_state;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};

use arboard::Clipboard;
use device_query::{DeviceEvents, DeviceQuery, DeviceState, Keycode};
use rand::Rng;
use rdev::{simulate, EventType, Key};
use serde_derive::{Deserialize, Serialize};
use tauri::api::path::{app_local_data_dir, config_dir};
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use tokio;
use toml::map::Map;
use toml::Value;
use walkdir::WalkDir;

#[macro_use]
extern crate lazy_static;
#[derive(Serialize, Deserialize, Debug)]
struct GifList {
    giflist: Vec<Link>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Link {
    link: String,
}

lazy_static! {
    static ref STOP_SIGNAL: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
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
async fn new_keys(emotion: &str) -> Result<String, String> {
    let _ = rmv_key(emotion);
    let device_state = DeviceState::new();
    let mut key_pressed: Vec<Keycode> = Vec::new();
    let mut last_keys: Vec<Keycode> = Vec::new();

    loop {
        let keys = device_state.get_keys();

        for key in &keys {
            if !last_keys.contains(key) && !key_pressed.contains(key) {
                key_pressed.push(*key);
            }
        }

        if keys.is_empty() && !key_pressed.is_empty() {
            break;
        }

        last_keys = keys.clone();
        tokio::time::sleep(Duration::from_millis(16)).await;
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
fn fetch_emo_key(emotion: &str) -> Result<String, String> {
    let file_content = read_toml()?;
    let mut get_config_val: toml::value::Table = toml::from_str(&file_content).unwrap();

    if let Some(settings) = get_config_val.get_mut("settings") {
        if let Some(value) = settings.get(emotion) {
            if let Some(val_str) = value.as_str() {
                return Ok(val_str.to_string());
            } else {
                return Ok(format!("").to_string());
            }
        }
    }

    Err("Ø".to_string())
}

#[tauri::command]
fn new_emo(name: &str) -> Result<(), String> {
    let _ = get_n_read(name);
    Ok(())
}

#[tauri::command]
fn rmv_emo(emotion: &str) -> Result<(), String> {
    let emotions_dir = get_local_data_path()?;
    let file_path = emotions_dir.join(format!("{}.json", emotion));
    println!("{:?}", file_path);

    if let Err(e) = fs::remove_file(&file_path) {
        return Err(format!("Failed to remove file: {}", e));
    }

    let _ = rmv_key(emotion);
    Ok(())
}
#[tauri::command]
fn watch_emo_folder() -> Result<String, String> {
    let emo_vec = get_emo_dir()?;
    Ok(serde_json::to_string(&emo_vec).unwrap())
}

fn main() {
    //config
    // let _ = watch_emo_folder();

    //tray
    let leave_item = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(leave_item)
        .add_native_item(SystemTrayMenuItem::Separator);

    // keys
    // let _ = listen_keys();

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
        .invoke_handler(tauri::generate_handler![
            add_to_list,
            new_keys,
            new_emo,
            fetch_emo_key,
            watch_emo_folder,
            rmv_emo
        ])
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

fn listen_keys(stop_signal: Arc<AtomicBool>) -> Result<(), String> {
    let device_state = Arc::new(DeviceState::new());

    let file_content = read_toml()?;

    let initial_config: toml::value::Table = toml::from_str(&file_content).unwrap();

    let config = Arc::new(Mutex::new(initial_config));

    let settings = config
        .lock()
        .unwrap()
        .get("settings")
        .unwrap()
        .as_table()
        .unwrap()
        .clone();
    println!("//// {:?}", settings);

    for (emotion, keycode_str) in settings.iter() {
        let emotion_clone = emotion.clone();
        let keycode_str_clone = keycode_str.clone();
        let device_state_clone = device_state.clone();
        let config_clone = config.clone();
        let stop_signal = stop_signal.clone();
        thread::spawn(move || {
            let _keycode = Keycode::from_str(keycode_str_clone.as_str().unwrap()).unwrap();
            let _guard = device_state_clone.on_key_down(move |key| {
                let config_guard = config_clone.lock().unwrap();
                let settings = config_guard.get("settings").unwrap().as_table().unwrap();
                let updated_keycode_str = settings.get(&emotion_clone).unwrap();

                let updated_keycode =
                    Keycode::from_str(updated_keycode_str.as_str().unwrap()).unwrap();

                if key == &updated_keycode {
                    // println!("{}", emotion_clone);
                    copy_pasta(&emotion_clone);
                }
            });
            loop {
                if stop_signal.load(Ordering::SeqCst) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });
    }

    Ok(())
}

fn save_config(config: &Map<String, Value>, stop_signal: Arc<AtomicBool>) -> Result<(), String> {
    let path = fetch_keyset()?;
    let toml_str = toml::to_string(&config).unwrap();

    File::create(path)
        .unwrap()
        .write_all(toml_str.as_bytes())
        .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(50));
    let _ = listen_keys(Arc::clone(&stop_signal));

    Ok(())
}

fn get_emo_dir() -> Result<Vec<String>, String> {
    let emotion_dir = get_local_data_path()?;
    let mut emo_box = Vec::new();

    for entry in WalkDir::new(emotion_dir) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let path = Path::new(entry.path());
            if let Some(stem) = path.file_stem() {
                if let Some(emo_name) = stem.to_str() {
                    emo_box.push(emo_name.to_string());
                }
            }
        }
    }

    let file_content = read_toml()?;
    let config: toml::value::Table = toml::from_str(&file_content).unwrap();

    STOP_SIGNAL.store(false, Ordering::SeqCst);
    let _ = save_config(&config, Arc::clone(&STOP_SIGNAL));

    Ok(emo_box)
}

fn fetch_keyset() -> Result<PathBuf, String> {
    let config_path = config_dir().ok_or_else(|| "Failed to get config directory".to_string())?;
    let trgif_roaming_dir = config_path.join(format!("TRGif"));
    if !trgif_roaming_dir.exists() {
        create_dir_all(&trgif_roaming_dir).map_err(|e| e.to_string())?;
    }
    let toml_file = trgif_roaming_dir.join(format!("config.toml"));

    if !toml_file.exists() {
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
            .open(&toml_file)
            .and_then(|mut file| file.write_all(toml_str.as_bytes()))
            .map_err(|e| e.to_string())?;
    }

    Ok(toml_file)
}

fn read_toml() -> Result<String, String> {
    let toml_file = fetch_keyset()?;
    let mut file_content = String::new();

    File::open(toml_file)
        .expect("Fichier non trouvé!")
        .read_to_string(&mut file_content)
        .expect("Erreur lors de la lecture du fichier");

    Ok(file_content)
}

fn insert_key(emotion: &str, value: &str) -> Result<(), String> {
    let file_content = read_toml()?;
    let mut config_add: toml::value::Table = toml::from_str(&file_content).unwrap();
    let key = &emotion;
    let value = &value;

    if let Some(db_map) = config_add
        .get_mut("settings")
        .and_then(|v| v.as_table_mut())
    {
        db_map.insert(key.to_string(), Value::String(value.to_string()));
    }

    STOP_SIGNAL.store(false, Ordering::SeqCst);
    save_config(&config_add, Arc::clone(&STOP_SIGNAL))
}

fn rmv_key(emotion: &str) -> Result<(), String> {
    let file_content = read_toml()?;
    let mut config_rmv: toml::value::Table = toml::from_str(&file_content).unwrap();
    let key = emotion;

    if let Some(db_map) = config_rmv
        .get_mut("settings")
        .and_then(|v| v.as_table_mut())
    {
        db_map.remove(key);
    }
    STOP_SIGNAL.store(true, Ordering::SeqCst);
    save_config(&config_rmv, Arc::clone(&STOP_SIGNAL))
}
