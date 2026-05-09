# ESP32 WiFi Scanner

`no_std` · `Embassy` · `esp-hal` · `Rust`

An async WiFi scanner and monitor application running on ESP32, written in Rust. Scans for nearby WiFi networks using concurrent Embassy tasks and outputs results via UART.

---

## Features

- **Async/await** > based task management (Embassy executor)
- **WiFi scanning** > Periodically scans for nearby networks and lists SSIDs
- **WiFi monitoring** > Checks connection state and automatically restarts if it drops
- **UART output** > All status messages are written to serial at 115200 baud
- **Mutex protection** > WiFi and UART resources are shared between tasks via `NoopRawMutex`
- **`no_std`** > No standard library; optimized for embedded systems

---

## Tasks

| Task | Description | Interval |
|---|---|---|
| `wifi_is_active` | Checks WiFi state, restarts it if inactive | 5 seconds |
| `wifi_scanner` | Scans up to 5 networks and prints SSIDs over UART | 10 seconds |
| `main` loop | Main loop — signals that the system is running | 1 second |

---

## Dependencies

| Crate | Purpose |
|---|---|
| `esp-hal` | ESP32 hardware abstraction layer |
| `embassy-executor` | Async task executor |
| `embassy-time` | Async timer |
| `embassy-sync` | Mutex and synchronization primitives |
| `esp-radio` | WiFi/BLE controller |
| `esp-alloc` | Heap allocator |
| `esp-rtos` | RTOS runtime |
| `esp-bootloader-esp-idf` | Bootloader integration |
| `heapless` | Fixed-size data structures on the stack |
| `static-cell` | Static lifetime variable initializer |

---

## Build & Flash

> Requires [espup](https://github.com/esp-rs/espup) and the Rust ESP toolchain to be installed first.

```bash
# Install toolchain (once)
espup install

# Build the project
cargo build --release

# Flash to ESP32 (requires espflash)
cargo run --release
```

To monitor serial output:

```bash
espflash monitor
```

---

## Example UART Output

```
Loop running.
WiFi is active.
SSID: Home_Wifi
SSID: Neighbor_Net
SSID: AndroidAP
WiFi is active.
Loop running.
```

---

## Architecture

```
main()
 ├── UART mutex initialized
 ├── Heap allocator (98KB + 72KB)
 ├── WiFi controller initialized (Client mode)
 ├── WiFi mutex initialized
 ├── [task] wifi_is_active  →  connection check every 5s
 └── [task] wifi_scanner    →  network scan every 10s
```

---

## Notes

- The panic handler enters an empty loop (`loop {}`); consider adding a log or reset mechanism for production use.
- `scan_with_config` is a blocking call — other tasks may be delayed during long scans.
- Both tasks acquire mutexes in the same order (WiFi first, then UART); this ordering must stay consistent to avoid deadlocks.

---

## License

MIT