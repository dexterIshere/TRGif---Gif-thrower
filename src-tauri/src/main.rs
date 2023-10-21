// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::OpenOptions;
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, thread};

use arboard::Clipboard;
use device_query::{DeviceEvents, DeviceQuery, DeviceState, Keycode};
use rand::Rng;
use rdev::{simulate, EventType, Key};
use serde_derive::{Deserialize, Serialize};
use tauri::api::path::app_local_data_dir;
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};

#[derive(Serialize, Deserialize, Debug)]
struct GifList {
    giflist: Vec<Link>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Link {
    link: String,
}

#[tauri::command]
fn get_n_read(emotion: &str) -> Result<(PathBuf, String), String> {
    let binding = tauri::generate_context!();
    let config = binding.config();
    let local_data_path = app_local_data_dir(&config)
        .ok_or_else(|| "Failed to get local data directory".to_string())?;
    let emotions_dir = local_data_path.join("emotions");
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
fn new_keys() -> Result<String, String> {
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

    Ok(new_cmd)
}

#[tauri::command]
async fn new_emo(name: &str) -> Result<(), String> {
    let _ = get_n_read(name);
    Ok(())
}

fn main() {
    //tary
    let leave_item = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(leave_item)
        .add_native_item(SystemTrayMenuItem::Separator);

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
