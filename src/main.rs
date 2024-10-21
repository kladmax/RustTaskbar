use std::process::Command;

fn main() {
    // Включаємо режим гібернації
    let output = Command::new("cmd")
        .args(&["/C", "powercfg -hibernate on"])
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        println!("Hibernation enabled successfully.");
    } else {
        eprintln!("Failed to enable hibernation.");
    }

    // Виконуємо команду для гібернації
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
