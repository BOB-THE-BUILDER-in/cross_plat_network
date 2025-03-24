use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use std::time::Duration;

// Resource to hold WebSocket state
#[derive(Resource)]
struct WsState {
    sender: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<String>>>>,
    messages: Arc<Mutex<Vec<String>>>,
    runtime: Runtime,
    last_update: Arc<Mutex<std::time::Instant>>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .insert_resource(WsState {
            sender: Arc::new(Mutex::new(None)),
            messages: Arc::new(Mutex::new(Vec::new())),
            runtime: tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime"),
            last_update: Arc::new(Mutex::new(std::time::Instant::now())),
        })
        .add_systems(Startup, setup_websocket)
        .add_systems(Update, ui_system.before(ui_update_system))
        .add_systems(Update, ui_update_system)
        .run();
}

fn setup_websocket(mut ws_state: ResMut<WsState>) {
    let url = "ws://127.0.0.1:8080";
    let messages = ws_state.messages.clone();
    let sender = ws_state.sender.clone();
    let last_update = ws_state.last_update.clone();
    let runtime = &ws_state.runtime;

    runtime.spawn(async move {
        let (ws_stream, _) = match connect_async(url).await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to connect to server: {}", e);
                return;
            }
        };
        println!("Connected to {}", url);

        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        {
            let mut sender_guard = sender.lock().unwrap();
            *sender_guard = Some(tx);
        }

        let send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = write.send(Message::Text(msg.into())).await {
                    eprintln!("Error sending message: {}", e);
                    break;
                }
            }
        });

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let mut messages_guard = messages.lock().unwrap();
                    if messages_guard.len() >= 100 {
                        messages_guard.remove(0);
                    }
                    messages_guard.push(format!("Received: {}", text));
                    
                    let mut last_update_guard = last_update.lock().unwrap();
                    *last_update_guard = std::time::Instant::now();
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    break;
                }
            }
        }

        send_task.abort();
    });
}

fn ui_system(mut contexts: EguiContexts, ws_state: Res<WsState>) {
    let ctx = contexts.ctx_mut();

    egui::Window::new("WebSocket Chat")
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Message:");
                static mut INPUT_MESSAGE: String = String::new(); // Persistent input
                ui.text_edit_singleline(unsafe { &mut INPUT_MESSAGE });
                
                if ui.button("Send").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let message = unsafe { INPUT_MESSAGE.clone() };
                    if !message.is_empty() {
                        let sender_guard = ws_state.sender.lock().unwrap();
                        if let Some(ref tx) = *sender_guard {
                            if tx.send(message.clone()).is_ok() {
                                let mut messages_guard = ws_state.messages.lock().unwrap();
                                messages_guard.push(format!("Sent: {}", message));
                                unsafe { INPUT_MESSAGE.clear() };
                            } else {
                                eprintln!("Failed to send message to channel");
                            }
                        } else {
                            eprintln!("Sender not initialized!");
                        }
                    }
                }
            });
        });
}

// Unchanged Messages window system
fn ui_update_system(mut contexts: EguiContexts, ws_state: Res<WsState>) {
    let ctx = contexts.ctx_mut();

    egui::Window::new("Messages")
        .resizable(true)
        .show(ctx, |ui| {
            let messages_guard = ws_state.messages.lock().unwrap();
            ui.label("Messages:");
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for msg in messages_guard.iter() {
                        ui.label(msg);
                    }
                });
        });
}