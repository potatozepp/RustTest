# RustTest
Simple command line tool for interacting with a serial COM port.

## Building

```bash
cargo build --release
```

## Running

Run the application and follow the prompts to select a serial port. Then type
commands and press Enter to send them. Responses from the device are printed
to the screen.

When connected you can use the **Disconnect** button to close the current
port and choose another one.

```bash
cargo run --release
```
