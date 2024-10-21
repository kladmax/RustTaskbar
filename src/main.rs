use eframe::{egui, epi};
use std::process::Command;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        Box::new(MyApp::default()),
        options,
    );
}

struct MyApp;

impl Default for MyApp {
    fn default() -> Self {
        Self
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "Hibernate Task"
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hibernate Task");
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
