// main.rs

#![windows_subsystem = "windows"]

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::System::Power::{
    SetThreadExecutionState, 
    ES_CONTINUOUS, 
    ES_DISPLAY_REQUIRED, 
    ES_SYSTEM_REQUIRED, 
    ES_AWAYMODE_REQUIRED,
    EXECUTION_STATE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VIRTUAL_KEY, VK_LBUTTON, VK_RBUTTON};

pub const KEY_PRESSED: i32 = 0x8000;
pub const MAX_IDLE_TIME: u32 = 60;

fn main() {
    let options = eframe::NativeOptions::default();
    let window_title = "Hibernate Task";
    
    eframe::run_native(
        window_title,
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    ).expect("Failed to start the application");
}

pub struct MyApp {
    idle_time: u32,
    timer_active: bool,
    timer_sender: Option<mpsc::Sender<()>>,
    idle_timer: Option<IdleTimer>,
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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hibernate Task");

            ui.horizontal(|ui| {
                ui.label("Idle Time (minutes):");
                ui.add(egui::Slider::new(&mut self.idle_time, 0..=MAX_IDLE_TIME));
            });

            let button_size = egui::vec2(200.0, 40.0);

            if ui
                .add_sized(
                    button_size,
                    egui::Button::new("Set Timer")
                        .fill(if self.timer_active { egui::Color32::GREEN } else { egui::Color32::RED }),
                )
                .clicked()
            {
                if self.timer_active {
                    if let Some(sender) = self.timer_sender.take() {
                        sender.send(()).ok();
                    }
                    self.timer_active = false;
                } else {
                    let (sender, receiver) = mpsc::channel();
                    self.timer_sender = Some(sender);
                    let idle_duration = Duration::from_secs((self.idle_time * 60) as u64);
                    self.timer_active = true;
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
                        idle_timer.stop();
                    }
                    self.idle_timer = None;
                } else {
                    let idle_duration = Duration::from_secs((self.idle_time * 60) as u64);
                    let mut idle_timer = IdleTimer::new(idle_duration);
                    idle_timer.start();
                    self.idle_timer = Some(idle_timer);
                }
            }

            if ui.add_sized(button_size, egui::Button::new("Run Hibernate")).clicked() {
                run_hibernate();
            }
        });
    }
}

pub struct IdleTimer {
    duration: Duration,
    sender: Option<mpsc::Sender<()>>,
}

impl IdleTimer {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            sender: None,
        }
    }

    pub fn start(&mut self) {
        self.sender = Some(start_idle_timer(self.duration, run_hibernate));
    }

    pub fn stop(&mut self) {
        if let Some(sender) = self.sender.take() {
            sender.send(()).ok();
        }
    }
}

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

pub fn start_idle_timer<F>(duration: Duration, run_hibernate: F) -> mpsc::Sender<()>
where
    F: Fn() + Send + 'static,
{
    let (sender, receiver) = mpsc::channel();
    
    thread::spawn(move || {
        let mut last_interaction = Instant::now();
        let mut display_active = false;
        
        // Встановлюємо початковий стан для запобігання сну
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
        }
        
        loop {
            // Перевіряємо активність користувача
            if is_user_active() {
                last_interaction = Instant::now();
                display_active = true;
            }
            
            // Перевіряємо стан дисплея кожні 5 секунд
            if Instant::now().duration_since(last_interaction).as_secs() % 5 == 0 {
                unsafe {
                    // Зберігаємо попередній стан
                    let prev_state = SetThreadExecutionState(ES_CONTINUOUS);
                    
                    // Якщо отримали нульовий стан - дисплей неактивний
                    if prev_state == EXECUTION_STATE(0) {
                        display_active = false;
                    } else {
                        display_active = true;
                        last_interaction = Instant::now();
                    }
                    
                    // Відновлюємо попередній стан
                    SetThreadExecutionState(
                        ES_CONTINUOUS | 
                        ES_DISPLAY_REQUIRED | 
                        ES_SYSTEM_REQUIRED |
                        ES_AWAYMODE_REQUIRED
                    );
                }
            }
            
            // Перевіряємо час простою
            if !display_active && Instant::now().duration_since(last_interaction) >= duration {
                run_hibernate();
                return;
            }
            
            // Перевіряємо сигнал зупинки
            if receiver.try_recv().is_ok() {
                return;
            }
            
            thread::sleep(Duration::from_secs(1));
        }
    });
    
    sender
}

pub fn keep_system_awake() {
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED);
    }
}

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