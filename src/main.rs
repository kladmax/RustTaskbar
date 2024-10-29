// Забезпечує, що програма працює без створення додаткового консольного вікна в Windows.
#![windows_subsystem = "windows"]

// Імпортуємо модулі для графічного інтерфейсу (eframe та egui) і конфігураційний файл.
use eframe::NativeOptions;
use crate::ui::MyApp;

mod ui;
mod logic;
mod config;
mod idle_timer;


fn main() {
    // Створюємо стандартні налаштування для eframe.
    let options = NativeOptions::default();

    // Запускаємо графічний інтерфейс, передаємо у програму структуру MyApp.
    let window_title = "Hibernate Task";
    eframe::run_native(
        window_title,
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    ).expect("Failed to start the application");
}
