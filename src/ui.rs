// ui.rs

use eframe::{egui, App};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use crate::logic::run_hibernate;
use crate::config::MAX_IDLE_TIME;
use crate::idle_timer::IdleTimer;

// Структура, що описує стан програми та параметри інтерфейсу.
pub struct MyApp {
    idle_time: u32,                  // Час простою в хвилинах для гібернації
    timer_active: bool,              // Вказує, чи активний таймер
    timer_sender: Option<mpsc::Sender<()>>, // Канал для зупинки таймера
    idle_timer: Option<IdleTimer>,   // Таймер для бездіяльності
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            idle_time: 0,
            timer_active: false,
            timer_sender: None,
            idle_timer: None,
        }
    }
}

// Реалізація логіки UI
impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hibernate Task"); // Назва додатку
            // Елемент управління для вибору часу простою
            ui.horizontal(|ui| {
                ui.label("Idle Time (minutes):");
                ui.add(egui::Slider::new(&mut self.idle_time, 0..=MAX_IDLE_TIME));
            });
            // Розмір для кнопок
            let button_size = egui::vec2(200.0, 40.0);

            // Кнопка для встановлення або скидання таймера
            if ui.add_sized(button_size, egui::Button::new("Set Timer")
                    .fill(if self.timer_active { egui::Color32::GREEN } else { egui::Color32::RED }))
                .clicked()
            {
                if self.timer_active {
                    if let Some(sender) = self.timer_sender.take() {
                        sender.send(()).ok();  // Зупиняємо таймер, надсилаючи повідомлення через канал
                    }
                    self.timer_active = false;
                } else {
                    let (sender, receiver) = mpsc::channel();
                    self.timer_sender = Some(sender);
                    let idle_duration = Duration::from_secs((self.idle_time * 60) as u64);
                    self.timer_active = true;
                    // Створюємо новий потік для таймера
                    thread::spawn(move || {
                        let start_time = Instant::now();
                        while Instant::now().duration_since(start_time) < idle_duration {
                            thread::sleep(Duration::from_secs(1));
                            if receiver.try_recv().is_ok() {
                                return;
                            }
                        }
                        run_hibernate();
                    });
                }
            }

            // Третя кнопка для бездіяльності
            if ui.add_sized(button_size, egui::Button::new("Idle Timer")
                    .fill(if self.idle_timer.is_some() { egui::Color32::YELLOW } else { egui::Color32::GRAY }))
                .clicked()
            {
                if self.idle_timer.is_some() {
                    if let Some(idle_timer) = &mut self.idle_timer {
                        idle_timer.stop();  // Зупиняємо таймер бездіяльності
                    }
                    self.idle_timer = None;
                } else {
                    let idle_duration = Duration::from_secs((self.idle_time * 60) as u64);
                    let mut idle_timer = IdleTimer::new(idle_duration);
                    idle_timer.start();  // Запускаємо таймер бездіяльності
                    self.idle_timer = Some(idle_timer);
                }
            }
             // Додаткова кнопка для негайного запуску гібернації
             if ui.add_sized(button_size, egui::Button::new("Run Hibernate")).clicked() {
                run_hibernate();
            }
        });
    }
}
