use eframe::{egui, epi};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

struct MyApp {
    idle_time: u32,
    timer_sender: Option<mpsc::Sender<()>>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            idle_time: 0,
            timer_sender: None,
        }
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "Hibernate Task"
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hibernate Task");

            ui.horizontal(|ui| {
                ui.label("Idle Time (minutes):");
                ui.add(egui::Slider::new(&mut self.idle_time, 0..=60));
            });

            if ui.button("Set Timer").clicked() {
                let (sender, receiver) = mpsc::channel();
                self.timer_sender = Some(sender);

                let idle_duration = Duration::from_secs((self.idle_time * 60) as u64);

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

            if ui.button("Run Hibernate").clicked() {
                run_hibernate();
            }
        });
    }
}

fn run_hibernate() {
    let output = Command::new("cmd")
        .args(&["/C", "powercfg -hibernate on"])
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        println!("Hibernation enabled successfully.");
    } else {
        eprintln!("Failed to enable hibernation.");
    }

    let output = Command::new("cmd")
        .args(&["/C", "shutdown /h"])
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        println!("System hibernating...");
    } else {
        eprintln!("Failed to hibernate system.");
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        Box::new(MyApp::default()),
        options,
    );
}
