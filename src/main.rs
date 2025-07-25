use std::io::{self, Read, Write};
use std::time::Duration;
use eframe::egui;

struct GuiApp {
    port: Box<dyn serialport::SerialPort>,
    input: String,
    output: String,
}

impl GuiApp {
    fn new(port: Box<dyn serialport::SerialPort>) -> Self {
        Self {
            port,
            input: String::new(),
            output: String::new(),
        }
    }

    fn send_command(&mut self) {
        let cmd = self.input.trim_end();
        if cmd.is_empty() {
            return;
        }
        if let Err(e) = self.port.write_all(cmd.as_bytes()) {
            self.output.push_str(&format!("Error sending: {e}\n"));
            return;
        }
        if let Err(e) = self.port.write_all(b"\r\n") {
            self.output.push_str(&format!("Error sending newline: {e}\n"));
            return;
        }

        let mut response = String::new();
        let mut buf = [0u8; 1];
        loop {
            match self.port.read(&mut buf) {
                Ok(1) => {
                    if buf[0] == b'\n' {
                        break;
                    }
                    response.push(buf[0] as char);
                }
                Ok(_) => break,
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => break,
                Err(e) => {
                    self.output.push_str(&format!("Read error: {e}\n"));
                    return;
                }
            }
        }
        if !response.is_empty() {
            self.output.push_str(&format!("< {}\n", response.trim_end()));
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let resp = ui.text_edit_singleline(&mut self.input);
                if ui.button("Send").clicked()
                    || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                {
                    self.send_command();
                    self.input.clear();
                }
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(&self.output);
            });
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Available serial ports:");
    match serialport::available_ports() {
        Ok(ports) => {
            for p in ports {
                println!("  {}", p.port_name);
            }
        }
        Err(e) => {
            eprintln!("Failed to list serial ports: {}", e);
        }
    }

    print!("Enter port name: ");
    io::stdout().flush()?;
    let mut port_name = String::new();
    io::stdin().read_line(&mut port_name)?;
    let port_name = port_name.trim();

    let port = serialport::new(port_name, 9600)
        .timeout(Duration::from_millis(2000))
        .open()?;

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Serial GUI",
        native_options,
        Box::new(|_cc| Box::new(GuiApp::new(port))),
    )?;
    Ok(())
}
