use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{delay::FreeRtos, prelude::Peripherals, units::FromValueType},
    http::{self},
    io::{EspIOError, Write as _},
    sys::c_camera::{
        camera_fb_location_t_CAMERA_FB_IN_PSRAM, camera_grab_mode_t_CAMERA_GRAB_WHEN_EMPTY,
        framesize_t_FRAMESIZE_UXGA, ledc_channel_t_LEDC_CHANNEL_0, ledc_timer_t_LEDC_TIMER_0,
        pixformat_t_PIXFORMAT_JPEG,
    },
    wifi::{self},
};
use log::{info, warn};
use rusty_esp32_cam::Camera;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    let wifi = wifi::EspWifi::new(peripherals.modem, sysloop.clone(), None)?;
    let mut wifi = wifi::BlockingWifi::wrap(wifi, sysloop)?;

    wifi.set_configuration(&wifi::Configuration::Client(
        wifi::ClientConfiguration::default(),
    ))?;

    info!("Starting WiFi...");
    wifi.start()?;

    info!("Scanning...");
    let ap_infos = wifi.scan()?;
    let channel = if let Some(ap) = ap_infos.into_iter().find(|ap| ap.ssid == CONFIG.wifi_ssid) {
        info!("Access point found on channel {:?}", ap.channel);
        Some(ap.channel)
    } else {
        warn!("Cannot find access point");
        None
    };

    wifi.set_configuration(&wifi::Configuration::Client(wifi::ClientConfiguration {
        ssid: CONFIG.wifi_ssid.try_into().unwrap(),
        password: CONFIG.wifi_psk.try_into().unwrap(),
        auth_method: wifi::AuthMethod::WPA2WPA3Personal,
        channel,
        ..Default::default()
    }))?;

    info!("Connecting to WiFi...");
    wifi.connect()?;

    info!("Waiting for DHCP lease...");
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("Wifi DHCP info: {:?}", ip_info);

    info!("Creating camera instance...");
    let camera = Camera::new(
        peripherals.pins.gpio32,
        peripherals.pins.gpio0,
        peripherals.pins.gpio35,
        peripherals.pins.gpio34,
        peripherals.pins.gpio39,
        peripherals.pins.gpio36,
        peripherals.pins.gpio21,
        peripherals.pins.gpio19,
        peripherals.pins.gpio18,
        peripherals.pins.gpio5,
        peripherals.pins.gpio25,
        peripherals.pins.gpio23,
        peripherals.pins.gpio22,
        peripherals.pins.gpio26,
        peripherals.pins.gpio27,
        20_u32.MHz().into(),
        ledc_timer_t_LEDC_TIMER_0,
        ledc_channel_t_LEDC_CHANNEL_0,
        pixformat_t_PIXFORMAT_JPEG,
        framesize_t_FRAMESIZE_UXGA, // QVGA || XGA || UXGA
        10,
        1,
        camera_fb_location_t_CAMERA_FB_IN_PSRAM,
        camera_grab_mode_t_CAMERA_GRAB_WHEN_EMPTY,
    )?;

    let mut server = http::server::EspHttpServer::new(&http::server::Configuration::default())?;

    server.fn_handler("/", http::Method::Get, move |request| {
        camera.get_framebuffer();
        // take two frames to get a fresh one
        let framebuffer = camera.get_framebuffer();

        if let Some(framebuffer) = framebuffer {
            let data = framebuffer.data();

            let headers = [
                ("Content-Type", "image/jpeg"),
                ("Content-Length", &data.len().to_string()),
            ];
            let mut response = request.into_response(200, Some("OK"), &headers)?;
            response.write_all(data)?;
        } else {
            let mut response = request.into_ok_response()?;
            response.write_all("no framebuffer".as_bytes())?;
        }

        Ok::<(), EspIOError>(())
    })?;

    loop {
        FreeRtos::delay_ms(1_000u32);
    }
}
