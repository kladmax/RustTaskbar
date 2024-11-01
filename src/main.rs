// main.rs

// Відключаємо консольне вікно для Windows, запускаючи програму без нього
#![windows_subsystem = "windows"]

// Імпортуємо необхідні стандартні модулі та функції
use std::sync::mpsc; // Для обміну повідомленнями між потоками
use std::thread; // Для роботи з потоками
use std::time::{Duration, Instant}; // Для роботи з часом

// Імпортуємо Windows API для керування станом живлення та введенням з клавіатури і миші
use windows::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED, ES_AWAYMODE_REQUIRED,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VIRTUAL_KEY, VK_LBUTTON, VK_RBUTTON};

// Константи
pub const KEY_PRESSED: i32 = 0x8000; // Визначає, що клавіша натиснута
pub const MAX_IDLE_TIME: u32 = 60; // Максимальний час простою (хвилин)

fn main() {
    // Конфігурація параметрів вікна за замовчуванням
    let options = eframe::NativeOptions::default();
    let window_title = "Hibernate Task"; // Назва програми
    
    // Запуск програми з основним вікном
    eframe::run_native(
        window_title,
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    ).expect("Failed to start the application");
}

// Основна структура програми, яка зберігає налаштування та стан таймера
pub struct MyApp {
    idle_time: u32, // Час простою, встановлений користувачем
    timer_active: bool, // Прапорець активності таймера
    timer_sender: Option<mpsc::Sender<()>>, // Канал для керування таймером
    idle_timer: Option<IdleTimer>, // Таймер для перевірки простою користувача
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            idle_time: 0, // Початковий час простою
            timer_active: false, // Таймер неактивний за замовчуванням
            timer_sender: None,
            idle_timer: None,
        }
    }
}

// Основна логіка програми
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Центральна панель інтерфейсу користувача
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hibernate Task"); // Заголовок програми

            // Вибір часу простою
            ui.horizontal(|ui| {
                ui.label("Idle Time (minutes):");
                ui.add(egui::Slider::new(&mut self.idle_time, 0..=MAX_IDLE_TIME));
            });

            // Задаємо розмір кнопок
            let button_size = egui::vec2(200.0, 40.0);

            // Кнопка для встановлення таймера
            if ui
                .add_sized(
                    button_size,
                    egui::Button::new("Set Timer")
                        .fill(if self.timer_active { egui::Color32::GREEN } else { egui::Color32::RED }),
                )
                .clicked()
            {
                // Логіка активації/деактивації таймера
                if self.timer_active {
                    if let Some(sender) = self.timer_sender.take() {
                        sender.send(()).ok(); // Зупиняємо таймер
                    }
                    self.timer_active = false;
                } else {
                    // Налаштування і запуск нового таймера
                    let (sender, receiver) = mpsc::channel();
                    self.timer_sender = Some(sender);
                    let idle_duration = Duration::from_secs((self.idle_time * 60) as u64);
                    self.timer_active = true;
                    thread::spawn(move || {
                        let start_time = Instant::now();
                        // Запускаємо таймер до завершення встановленого часу або до зупинки
                        while Instant::now().duration_since(start_time) < idle_duration {
                            thread::sleep(Duration::from_secs(1));
                            if receiver.try_recv().is_ok() {
                                return;
                            }
                        }
                        run_hibernate(); // Запускаємо гібернацію після завершення таймера
                    });
                }
            }

            // Кнопка для запуску Idle Timer
            if ui
                .add_sized(
                    button_size,
                    egui::Button::new("Idle Timer")
                        .fill(if self.idle_timer.is_some() { egui::Color32::YELLOW } else { egui::Color32::GRAY }),
                )
                .clicked()
            {
                if self.idle_timer.is_some() {
                    if let Some(idle_timer) = &mut self.idle_timer {
                        idle_timer.stop(); // Зупиняємо активний таймер
                    }
                    self.idle_timer = None;
                } else {
                    // Налаштовуємо і запускаємо новий Idle Timer
                    let idle_duration = Duration::from_secs((self.idle_time * 60) as u64);
                    let mut idle_timer = IdleTimer::new(idle_duration);
                    idle_timer.start();
                    self.idle_timer = Some(idle_timer);
                }
            }

            // Кнопка для примусової гібернації
            if ui.add_sized(button_size, egui::Button::new("Run Hibernate")).clicked() {
                run_hibernate(); // Виконуємо команду гібернації
            }
        });
    }
}

// Структура IdleTimer зберігає налаштування для таймера простою
pub struct IdleTimer {
    duration: Duration, // Тривалість простою
    sender: Option<mpsc::Sender<()>>, // Канал для завершення таймера
}

impl IdleTimer {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            sender: None,
        }
    }

    pub fn start(&mut self) {
        // Запуск таймера простою з відстеженням активності
        self.sender = Some(start_idle_timer(self.duration, run_hibernate));
    }

    pub fn stop(&mut self) {
        // Зупинка таймера, надсилаємо сигнал завершення
        if let Some(sender) = self.sender.take() {
            sender.send(()).ok();
        }
    }
}

// Функція для перевірки активності користувача (миша або клавіатура)
pub fn is_user_active() -> bool {
    unsafe {
        let mouse_active =
            (GetAsyncKeyState(VK_RBUTTON.0.into()) as i32 & KEY_PRESSED) != 0 ||
            (GetAsyncKeyState(VK_LBUTTON.0.into()) as i32 & KEY_PRESSED) != 0;
        let key_active = (0x01..=0xFE)
            .map(|key| VIRTUAL_KEY(key))
            .any(|key| GetAsyncKeyState(key.0.into()) as i32 & KEY_PRESSED != 0);
        mouse_active || key_active
    }
}

// Запускає потік, що завершується через визначений час простою
pub fn start_idle_timer<F>(duration: Duration, run_hibernate: F) -> mpsc::Sender<()>
where
    F: Fn() + Send + 'static,
{
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut last_interaction = Instant::now();
        loop {
            if is_user_active() {
                last_interaction = Instant::now();
            }
            unsafe {
                SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
            }
            if Instant::now().duration_since(last_interaction) >= duration {
                run_hibernate();
                return;
            }
            if receiver.try_recv().is_ok() {
                return;
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
    sender
}

// Функція, що запобігає переходу системи в режим сну
pub fn keep_system_awake() {
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED);
    }
}

// Функція для гібернації системи
pub fn run_hibernate() {
    let output = std::process::Command::new("cmd")
        .args(&["/C", "powercfg -hibernate on"])
        .output()
        .expect("Failed to execute command");
    if output.status.success() {
        println!("Hibernation enabled successfully.");
    } else {
        eprintln!("Failed to enable hibernation.");
    }

    let output = std::process::Command::new("cmd")
        .args(&["/C", "shutdown /h"])
        .output()
        .expect("Failed to execute command");
    if output.status.success() {
        println!("System hibernating...");
    } else {
        eprintln!("Failed to initiate hibernation.");
    }
}
