// ui.rs

// Імпортуємо основні елементи для побудови UI
use eframe::{egui, epi};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use crate::logic::run_hibernate;
use crate::config::MAX_IDLE_TIME;

// Структура, що описує стан програми та параметри інтерфейсу.
pub struct MyApp {
    idle_time: u32,                  // Час простою в хвилинах для гібернації
    timer_active: bool,              // Вказує, чи активний таймер
    timer_sender: Option<mpsc::Sender<()>>, // Канал для зупинки таймера
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            idle_time: 0,
            timer_active: false,
            timer_sender: None,
        }
    }
}

// Реалізація логіки UI
impl epi::App for MyApp {
    fn name(&self) -> &str {
        "Hibernate Task"
    }

    // Оновлення UI для відображення та взаємодії з користувачем
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hibernate Task"); // Назва додатку

            // Елемент управління для вибору часу простою
            ui.horizontal(|ui| {
                ui.label("Idle Time (minutes):");
                ui.add(egui::Slider::new(&mut self.idle_time, 0..=MAX_IDLE_TIME));
            });

            // Кнопка для встановлення або скидання таймера
            if ui.add(egui::Button::new("Set Timer")
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

            // Додаткова кнопка для негайного запуску гібернації
            if ui.button("Run Hibernate").clicked() {
                run_hibernate();
            }
        });
    }
}
