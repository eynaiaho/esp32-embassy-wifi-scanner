# ESP32 WiFi Scanner

`no_std` · `Embassy` · `esp-hal` · `Rust`

Rust ile yazılmış, ESP32 üzerinde çalışan async WiFi tarayıcı ve izleme uygulaması. Embassy executor kullanarak eş zamanlı görevlerle WiFi ağlarını tarar ve UART üzerinden seri porta çıktı verir.

---

## Özellikler

- **Async/await** > tabanlı görev yönetimi (Embassy executor)
- **WiFi tarama** > Çevredeki ağları periyodik olarak tarar ve SSID'leri listeler
- **WiFi izleme** > Bağlantı durumunu kontrol eder, kopması durumunda otomatik yeniden başlatır
- **UART çıktısı** > Tüm durum mesajları 115200 baud üzerinden seri porta yazılır
- **Mutex koruması** > WiFi ve UART kaynakları görevler arasında `NoopRawMutex` ile paylaşılır
- **`no_std`** > Standart kütüphane kullanılmaz; gömülü sistem için optimize edilmiştir

---

## Görevler

| Görev | Açıklama | Periyot |
|---|---|---|
| `wifi_is_active` | WiFi durumunu kontrol eder, pasifse yeniden başlatır | 5 saniye |
| `wifi_scanner` | En fazla 5 ağ tarar, SSID'leri UART'a yazar | 10 saniye |
| `main` loop | Ana döngü — sistemin çalıştığını bildirir | 1 saniye |

---

## Bağımlılıklar

| Kütüphane | Amaç |
|---|---|
| `esp-hal` | ESP32 donanım soyutlama katmanı |
| `embassy-executor` | Async görev yöneticisi |
| `embassy-time` | Asenkron zamanlayıcı |
| `embassy-sync` | Mutex ve senkronizasyon primitifleri |
| `esp-radio` | WiFi/BLE kontrolcüsü |
| `esp-alloc` | Heap bellek ayırıcı |
| `esp-rtos` | RTOS runtime |
| `esp-bootloader-esp-idf` | Bootloader entegrasyonu |
| `heapless` | Stack üzerinde sabit boyutlu veri yapıları |
| `static-cell` | Statik ömürlü değişken başlatıcı |

---

## Derleme ve Yükleme

> Önce [espup](https://github.com/esp-rs/espup) ve Rust ESP toolchain kurulu olmalıdır.

```bash
# Toolchain kurulumu (bir kez yapılır)
espup install

# Projeyi derle
cargo build --release

# ESP32'ye yükle (espflash gerekli)
cargo run --release
```

Seri port çıktısını izlemek için:

```bash
espflash monitor
```

---

## 🖥️ Örnek UART Çıktısı

```
Dongu calisiyor.
WiFi calisiyor.
SSID: Ev_Wifi
SSID: Komsu_Net
SSID: AndroidAP
WiFi calisiyor.
Dongu calisiyor.
```

---

## Mimari

```
main()
 ├── UART mutex başlatılıyor
 ├── Heap allocator (98KB + 72KB)
 ├── WiFi controller başlatılıyor (Client modu)
 ├── WiFi mutex başlatılıyor
 ├── [task] wifi_is_active  →  her 5s bağlantı kontrolü
 └── [task] wifi_scanner    →  her 10s ağ taraması
```

---

## Dikkat Edilmesi Gerekenler

- Panic handler boş bir döngüye girer (`loop {}`); production için log veya reset mekanizması eklenebilir.
- `scan_with_config` senkron bir çağrıdır — uzun tarama sürelerinde diğer görevler gecikebilir.
- Her iki görev de aynı anda mutex bekleyebileceğinden kilitlenme (deadlock) riskine dikkat edilmelidir. Mevcut kodda her görev önce WiFi, sonra UART mutex'ini alıyor; bu sıranın tutarlı tutulması önemlidir.

---

## Lisans

MIT