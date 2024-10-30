// idle_timer.rs 

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use windows::Win32::System::Power::{SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VIRTUAL_KEY, VK_LBUTTON, VK_RBUTTON};
use crate::power_management;




const KEY_PRESSED: i32 = 0x8000; // Визначаємо константу для стану натискання клавіші

pub struct IdleTimer {
    duration: Duration,                // Час простою, після якого спрацьовує дія
    sender: Option<mpsc::Sender<()>>,  // Канал для зупинки таймера
}

impl IdleTimer {
    // Створює новий таймер простою з заданою тривалістю
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            sender: None,
        }
    }

    // Запуск таймера простою
    pub fn start(&mut self) {
        let (sender, receiver) = mpsc::channel();
        self.sender = Some(sender);
        let idle_duration = self.duration;

        // Запускаємо новий потік для відстеження простою
        thread::spawn(move || {
            let mut last_interaction = Instant::now();

            loop {
                // Перевірка активності введення користувача
                if is_user_active() {
                    last_interaction = Instant::now();
                }

                 // Підтримуємо систему у активному стані
                power_management::keep_system_awake();

                // Підтримуємо активний стан дисплея та системи
                unsafe {
                    SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
                }

                // Якщо час простою перевищено, запускаємо функцію гібернації
                if Instant::now().duration_since(last_interaction) >= idle_duration {
                    super::logic::run_hibernate();
                    return;
                }

                // Завершуємо цикл, якщо отримано сигнал зупинки
                if receiver.try_recv().is_ok() {
                    return;
                }

                // Затримка для перевірки кожну секунду
                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    // Зупинка таймера
    pub fn stop(&mut self) {
        if let Some(sender) = self.sender.take() {
            sender.send(()).ok();
        }
    }
}

// Функція для перевірки активності миші та клавіатури
fn is_user_active() -> bool {
    unsafe {
        // Перевірка лівої та правої кнопки миші
        let mouse_active =
            (GetAsyncKeyState(VK_RBUTTON.0.into()) as i32 & KEY_PRESSED) != 0 ||
            (GetAsyncKeyState(VK_LBUTTON.0.into()) as i32 & KEY_PRESSED) != 0;

        // Перевірка натискання клавіш на клавіатурі
        let key_active = (0x01..=0xFE)
            .map(|key| VIRTUAL_KEY(key))
            .any(|key| GetAsyncKeyState(key.0.into()) as i32 & KEY_PRESSED != 0);

        // Повертаємо true, якщо активна миша або клавіатура
        mouse_active || key_active
    }
}
