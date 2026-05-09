#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use esp_radio::wifi::WifiController;
use static_cell::StaticCell;
use esp_radio::Controller;
use esp_hal::uart::Uart;
use heapless::String;
use core::fmt::Write;
use esp_hal::Async;
use embedded_io_async::Write as WriteUart;
use esp_radio::wifi::ClientConfig;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]

#[embassy_executor::task]
async fn wifi_is_active(wifi: &'static Mutex<NoopRawMutex, WifiController<'static>>, uart: &'static Mutex<NoopRawMutex, Uart<'static, Async>>) {
    loop {
        {
            let mut wifi_controller: embassy_sync::mutex::MutexGuard<_, _> = wifi.lock().await;
            let mut uart_controller: embassy_sync::mutex::MutexGuard<_, _> = uart.lock().await; 
            if wifi_controller.is_started().expect("wifi_is_active function loop>if error | in loop line 2") {
                WriteUart::write(&mut *uart_controller, b"WiFi calisiyor. \r\n").await.ok();
            } else {
                WriteUart::write(&mut *uart_controller, b"WiFi baglantisi kesik. Tekrar baslatiliyor... \n").await.ok();
                let _ = wifi_controller.start_async().await;
                WriteUart::write(&mut *uart_controller, b"WiFi baglantisi saglandi. \r\n").await.ok();
            }
        }
        Timer::after(Duration::from_secs(5)).await;
    }
}

#[embassy_executor::task]
async fn wifi_scanner(wifi: &'static Mutex<NoopRawMutex, WifiController<'static>>, uart: &'static Mutex<NoopRawMutex, Uart<'static, Async>>) {
    let mut string_buffer: String<64> = String::new();
    loop {
        {
            let mut wifi_controller: embassy_sync::mutex::MutexGuard<_, _> = wifi.lock().await;
            let mut uart_controller: embassy_sync::mutex::MutexGuard<_, _> = uart.lock().await; 
            match wifi_controller.scan_with_config(esp_radio::wifi::ScanConfig::default().with_max(5)) {
                Ok(networks) => {
                    let _val = networks.len();
                    for network in networks {
                        string_buffer.clear();
                        write!(string_buffer, "SSID: {} \n", network.ssid).ok();
                        WriteUart::write(&mut *uart_controller, string_buffer.as_bytes()).await.ok();
                    }
                }
                Err(_) => {
                    WriteUart::write(&mut *uart_controller, b"Tarama hatasi").await.unwrap();
                }
            }
        }
        Timer::after(Duration::from_secs(10)).await;
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let raw_uart = esp_hal::uart::Uart::new(peripherals.UART0, esp_hal::uart::Config::default().with_baudrate(115200)).unwrap().into_async();
    let uart_mutex = Mutex::new(raw_uart);
    static UART_CELL: StaticCell<Mutex<NoopRawMutex, Uart<'static, Async>>> = StaticCell::new();
    let uart = UART_CELL.init(uart_mutex);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);
    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    static INIT_CELL: StaticCell<Controller> = StaticCell::new();

    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let radio_init_cell = INIT_CELL.init(radio_init);
    let (mut wifi_init, _interfaces) = esp_radio::wifi::new(radio_init_cell, peripherals.WIFI, Default::default()).expect("Failed to initialize Wi-Fi controller");
    wifi_init.set_config(&esp_radio::wifi::ModeConfig::Client(ClientConfig::default())).unwrap();
    let _ = wifi_init.start_async().await;
    
    let wifi_mtx = Mutex::new(wifi_init);

    static WIFI_CELL: StaticCell<Mutex<NoopRawMutex, WifiController<'static>>> = StaticCell::new();

    let wifi = WIFI_CELL.init(wifi_mtx);

    spawner.spawn(wifi_is_active(wifi, uart)).unwrap();
    spawner.spawn(wifi_scanner(wifi, uart)).unwrap();

    loop {
        {
            let mut uart_controller = uart.lock().await;
            WriteUart::write(&mut *uart_controller, b"Dongu calisiyor. \r\n").await.ok();
        }
        Timer::after(Duration::from_secs(1)).await;
    }
}
