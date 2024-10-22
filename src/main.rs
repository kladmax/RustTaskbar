#![windows_subsystem = "windows"] // Ця директива використовується для того, щоб при запуску програми не з'являлося консольне вікно CMD в Windows.

use eframe::{egui, epi}; // Імпортуємо бібліотеку eframe для створення графічного інтерфейсу.
use std::process::Command; // Імпортуємо Command для запуску системних команд.
use std::sync::mpsc; // Імпортуємо канал mpsc для обміну повідомленнями між потоками.
use std::thread; // Імпортуємо модуль для роботи з потоками.
use std::time::{Duration, Instant}; // Імпортуємо для вимірювання часу і затримок.

// Структура для збереження стану програми.
struct MyApp {
    idle_time: u32, // Кількість хвилин простою, після якого система має увійти в сплячий режим.
    timer_active: bool, // Вказує, чи активований таймер.
    timer_sender: Option<mpsc::Sender<()>>, // Канал для завершення таймера при натисканні кнопки.
}

// Реалізація стандартних налаштувань для MyApp.
impl Default for MyApp {
    fn default() -> Self {
        Self {
            idle_time: 0, // Початковий час простою — 0 хвилин.
            timer_active: false, // Таймер спочатку вимкнений.
            timer_sender: None, // Немає активного каналу для таймера.
        }
    }
}

// Реалізація функцій для програми, які вимагає eframe (epi::App).
impl epi::App for MyApp {
    fn name(&self) -> &str {
        // Назва програми, яка буде відображена в заголовку вікна.
        "Hibernate Task"
    }

    // Основна функція, яка викликається для оновлення UI.
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hibernate Task"); // Відображаємо заголовок.

            // Відображаємо горизонтальний контейнер з ярликом і слайдером для налаштування idle_time.
            ui.horizontal(|ui| {
                ui.label("Idle Time (minutes):"); // Текст ярлика.
                ui.add(egui::Slider::new(&mut self.idle_time, 0..=60)); // Слайдер для вибору часу простою (0-60 хвилин).
            });

            // Кнопка для встановлення таймера. Колір кнопки змінюється в залежності від того, активний таймер чи ні.
            if ui.add(egui::Button::new("Set Timer").fill(if self.timer_active { egui::Color32::GREEN } else { egui::Color32::RED })).clicked() {
                if self.timer_active {
                    // Якщо таймер активний, відправляємо сигнал завершення через канал.
                    if let Some(sender) = self.timer_sender.take() {
                        sender.send(()).ok(); // Відправляємо порожнє повідомлення, щоб зупинити таймер.
                    }
                    self.timer_active = false; // Деактивуємо таймер.
                } else {
                    // Якщо таймер неактивний, створюємо новий канал для управління таймером.
                    let (sender, receiver) = mpsc::channel();
                    self.timer_sender = Some(sender);

                    // Розраховуємо тривалість простою в секундах.
                    let idle_duration = Duration::from_secs((self.idle_time * 60) as u64);
                    self.timer_active = true; // Активуємо таймер.

                    // Створюємо новий потік для відліку часу.
                    thread::spawn(move || {
                        let start_time = Instant::now(); // Фіксуємо час початку таймера.
                        // Цикл, що перевіряє, чи минув заданий час простою.
                        while Instant::now().duration_since(start_time) < idle_duration {
                            thread::sleep(Duration::from_secs(1)); // Засипаємо на 1 секунду в кожній ітерації.
                            if receiver.try_recv().is_ok() {
                                // Якщо отримано сигнал через канал, перериваємо таймер.
                                return;
                            }
                        }
                        run_hibernate(); // Якщо час простою закінчився, запускаємо сплячий режим.
                    });
                }
            }

            // Кнопка для примусового переходу в сплячий режим.
            if ui.button("Run Hibernate").clicked() {
                run_hibernate(); // Запускаємо функцію для переходу в сплячий режим.
            }
        });
    }
}

// Функція для запуску команд, що переводять систему в сплячий режим.
fn run_hibernate() {
    // Спочатку активуємо режим гібернації.
    let output = Command::new("cmd")
        .args(&["/C", "powercfg -hibernate on"]) // Виконуємо команду активації режиму гібернації.
        .output()
        .expect("Failed to execute command"); // Якщо виникла помилка, виводимо її.

    if output.status.success() {
        // Якщо команда виконана успішно, повідомляємо про це.
        println!("Hibernation enabled successfully.");
    } else {
        // Якщо команда завершилася з помилкою, виводимо повідомлення про невдачу.
        eprintln!("Failed to enable hibernation.");
    }

    // Після активації режиму гібернації, викликаємо команду для переходу в сплячий режим.
    let output = Command::new("cmd")
        .args(&["/C", "shutdown /h"]) // Виконуємо команду для переходу в сплячий режим.
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        // Якщо команда виконана успішно, повідомляємо про це.
        println!("System hibernating...");
    } else {
        // Якщо команда завершилася з помилкою, виводимо повідомлення про невдачу.
        eprintln!("Failed to hibernate system.");
    }
}

// Головна функція програми.
fn main() {
    let options = eframe::NativeOptions::default(); // Створюємо стандартні налаштування для вікна програми.
    eframe::run_native(
        Box::new(MyApp::default()), // Запускаємо програму з використанням MyApp.
        options,
    );
}
