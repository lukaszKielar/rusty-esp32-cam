# Rusty ESP32 CAM

The purpose of this project was to create simple HTTP server streaming footage from ESP32's camera.

Project has been generated using [esp-rs/esp-idf-template](https://github.com/esp-rs/esp-idf-template) repository.
Make sure you follow [prerequisites](https://github.com/esp-rs/esp-idf-template?tab=readme-ov-file#prerequisites) sections before you start compiling the project.

Create `cfg.toml` file with credentials of your WiFi network. You can use existing template, but remember to update `wifi_ssid` and `wifi_psk` values:

```
cp cfg.toml.example cfg.toml
```

To compile and flash the board simply type:

```bash
cargo run --release
```

Hit enter and select serial port of the device. Wait until program is flashed and search for the IP of the board:

```bash
...
I (11960) esp_netif_handlers: sta ip: 192.168.0.X, mask: 255.255.255.0, gw: 192.168.0.1
...
```

Once finished, go to http://192.168.0.X and enjoy footage from your ESP32-CAM module.

## Acknowledgements

- https://github.com/Kezii/esp32cam_rs
