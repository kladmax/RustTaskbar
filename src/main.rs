// main.rs ініціалізує додаток.
// config.rs містить конфігурацію.
// ui.rs створює UI і керує його взаємодією.
// logic.rs запускає команди для переходу системи в режим гібернації.

// main.rs
// Цей файл ініціалізує програму, імпортує необхідні модулі та викликає основний цикл додатку.

// Забезпечує, що програма працює без створення додаткового консолевого вікна в Windows.
#![windows_subsystem = "windows"]

// Імпортуємо модулі для графічного інтерфейсу (eframe та egui) і конфігураційний файл.
use eframe::NativeOptions;
use crate::ui::MyApp;

mod ui;
mod logic;
mod config;

fn main() {
    // Створюємо стандартні налаштування для eframe.
    let options = NativeOptions::default();

    // Запускаємо графічний інтерфейс і передаємо у програму структуру MyApp.
    eframe::run_native(
        Box::new(MyApp::default()),
        options,
    );
}
