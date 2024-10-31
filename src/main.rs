// main.rs

// Забезпечує, що програма працює без створення додаткового консольного вікна в Windows.
#![windows_subsystem = "windows"]

mod ui;
mod logic;
mod config;
mod idle_button; 
mod idle_timer; // Імпорт нового модуля для таймера бездіяльності
mod power_management;

fn main() {
    let options = eframe::NativeOptions::default();
    let window_title = "Hibernate Task";
    eframe::run_native(
        window_title,
        options,
        Box::new(|_cc| Ok(Box::new(ui::MyApp::default()))),
    ).expect("Failed to start the application");
}

