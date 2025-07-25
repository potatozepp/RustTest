use std::io::{self, Read, Write};
use std::time::Duration;

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

    let mut port = serialport::new(port_name, 9600)
        .timeout(Duration::from_millis(2000))
        .open()?;

    println!("Type commands and press Enter. Empty line to quit.");
    let mut input = String::new();
    loop {
        print!("> ");
        io::stdout().flush()?;
        input.clear();
        if io::stdin().read_line(&mut input)? == 0 {
            break;
        }
        let cmd = input.trim_end();
        if cmd.is_empty() {
            break;
        }
        port.write_all(cmd.as_bytes())?;
        port.write_all(b"\r\n")?;

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
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                    break;
                }
                Err(e) => return Err(e.into()),
            }
        }
        if !response.is_empty() {
            println!("< {}", response.trim_end());
        }
    }
    Ok(())
}
