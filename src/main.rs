use std::io::{self, Read, Write};
use std::time::Duration;
use std::collections::VecDeque;
use eframe::egui;

#[derive(PartialEq, Eq, Clone, Copy)]
enum NewlineMode {
    None,
    CR,
    LF,
    CRLF,
}

impl NewlineMode {
    fn as_bytes(&self) -> &'static [u8] {
        match self {
            NewlineMode::None => b"",
            NewlineMode::CR => b"\r",
            NewlineMode::LF => b"\n",
            NewlineMode::CRLF => b"\r\n",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            NewlineMode::None => "None",
            NewlineMode::CR => "CR",
            NewlineMode::LF => "LF",
            NewlineMode::CRLF => "CRLF",
        }
    }
}

struct GuiApp {
    port: Option<Box<dyn serialport::SerialPort>>,
    ports: Vec<String>,
    selected_port: usize,
    input: String,
    output: VecDeque<String>,
    newline: NewlineMode,
    error: String,
}

impl GuiApp {
    fn new() -> Self {
        Self {
            port: None,
            ports: Self::available_ports(),
            selected_port: 0,
            input: String::new(),
            output: VecDeque::new(),
            newline: NewlineMode::CRLF,
            error: String::new(),
        }
    }

    fn available_ports() -> Vec<String> {
        match serialport::available_ports() {
            Ok(ports) => ports.into_iter().map(|p| p.port_name).collect(),
            Err(_) => Vec::new(),
        }
    }

    fn refresh_ports(&mut self) {
        self.ports = Self::available_ports();
        if self.selected_port >= self.ports.len() {
            self.selected_port = 0;
        }
    }

    fn open_selected_port(&mut self) {
        if let Some(name) = self.ports.get(self.selected_port).cloned() {
            match serialport::new(name, 9600)
                .timeout(Duration::from_millis(2000))
                .open() {
                Ok(p) => {
                    self.port = Some(p);
                    self.error.clear();
                }
                Err(e) => {
                    self.error = format!("Failed to open port: {e}");
                }
            }
        }
    }

    fn push_output(&mut self, line: String) {
        self.output.push_back(line);
        while self.output.len() > 100 {
            self.output.pop_front();
        }
    }

    fn send_command(&mut self) {
        if self.port.is_none() {
            return;
        }
        let cmd = self.input.trim_end();
        if cmd.is_empty() {
            return;
        }
        if let Some(port) = self.port.as_mut() {
            if let Err(e) = port.write_all(cmd.as_bytes()) {
                self.push_output(format!("Error sending: {e}"));
                return;
            }
            if let Err(e) = port.write_all(self.newline.as_bytes()) {
                self.push_output(format!("Error sending newline: {e}"));
                return;
            }

            self.push_output(format!("> {}", cmd));
            let mut response = String::new();
            let mut buf = [0u8; 1];
            loop {
                match port.read(&mut buf) {
                    Ok(1) => {
                        if buf[0] == b'\n' {
                            break;
                        }
                        response.push(buf[0] as char);
                    }
                    Ok(_) => break,
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => break,
                    Err(e) => {
                        self.push_output(format!("Read error: {e}"));
                        return;
                    }
                }
            }
            if !response.trim().is_empty() {
                self.push_output(format!("< {}", response.trim_end()));
            }
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.port.is_none() {
                ui.horizontal(|ui| {
                    if ui.button("Refresh").clicked() {
                        self.refresh_ports();
                    }
                    ui.label("Port:");
                    egui::ComboBox::from_id_source("port_select")
                        .selected_text(self.ports.get(self.selected_port).map(String::as_str).unwrap_or("-"))
                        .show_ui(ui, |ui| {
                            for (i, name) in self.ports.iter().enumerate() {
                                ui.selectable_value(&mut self.selected_port, i, name);
                            }
                        });
                    if ui.button("Open").clicked() {
                        self.open_selected_port();
                    }
                });
                if !self.error.is_empty() {
                    ui.colored_label(egui::Color32::RED, &self.error);
                }
            } else {
                let input_id = egui::Id::new("serial_input");
                let mut send = false;

                ui.horizontal(|ui| {
                    let resp = ui.add(egui::TextEdit::singleline(&mut self.input).id(input_id));
                    send |= ui.button("Send").clicked();
                    send |= resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    egui::ComboBox::from_id_source("newline")
                        .selected_text(self.newline.label())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.newline, NewlineMode::None, NewlineMode::None.label());
                            ui.selectable_value(&mut self.newline, NewlineMode::CR, NewlineMode::CR.label());
                            ui.selectable_value(&mut self.newline, NewlineMode::LF, NewlineMode::LF.label());
                            ui.selectable_value(&mut self.newline, NewlineMode::CRLF, NewlineMode::CRLF.label());
                        });
                });

                if send {
                    self.send_command();
                    self.input.clear();
                    ui.memory_mut(|mem| mem.request_focus(input_id));
                }

                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for line in &self.output {
                            ui.label(line);
                        }
                    });
            }
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Serial GUI",
        native_options,
        Box::new(|_cc| Box::new(GuiApp::new())),
    )?;
    Ok(())
}

