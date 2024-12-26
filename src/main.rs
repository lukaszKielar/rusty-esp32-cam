use std::marker::PhantomData;

use esp_idf_svc::{
    hal::{
        delay::FreeRtos,
        gpio::{self, Pin},
        prelude::Peripherals,
        units::{FromValueType, Hertz},
    },
    sys::{
        camera::{
            self, camera_fb_location_t_CAMERA_FB_IN_PSRAM,
            camera_grab_mode_t_CAMERA_GRAB_WHEN_EMPTY, framesize_t_FRAMESIZE_QVGA,
            ledc_channel_t_LEDC_CHANNEL_0, ledc_timer_t_LEDC_TIMER_0, pixformat_t_PIXFORMAT_JPEG,
        },
        esp, gpio_num_t_GPIO_NUM_NC, EspError,
    },
};
use log::{error, info};

struct Camera<'a> {
    _p: PhantomData<&'a ()>,
}

impl<'a> Camera<'a> {
    pub fn new(
        pin_pwdn: gpio::Gpio32,
        pin_xclk: gpio::Gpio0,
        pin_d7: gpio::Gpio35,
        pin_d6: gpio::Gpio34,
        pin_d5: gpio::Gpio39,
        pin_d4: gpio::Gpio36,
        pin_d3: gpio::Gpio21,
        pin_d2: gpio::Gpio19,
        pin_d1: gpio::Gpio18,
        pin_d0: gpio::Gpio5,
        pin_vsync: gpio::Gpio25,
        pin_href: gpio::Gpio23,
        pin_pclk: gpio::Gpio22,
        pin_sda: gpio::Gpio26,
        pin_scl: gpio::Gpio27,
        xclk_freq_hz: Hertz,
        ledc_timer: camera::ledc_timer_t,
        ledc_channel: camera::ledc_channel_t,
        pixel_format: camera::pixformat_t,
        frame_size: camera::framesize_t,
        jpeg_quality: i32,
        fb_count: usize,
        fb_location: camera::camera_fb_location_t,
        grab_mode: camera::camera_grab_mode_t,
    ) -> Result<Self, EspError> {
        let cam_config = camera::camera_config_t {
            pin_pwdn: pin_pwdn.pin(),
            pin_reset: gpio_num_t_GPIO_NUM_NC,
            pin_xclk: pin_xclk.pin(),

            pin_d7: pin_d7.pin(),
            pin_d6: pin_d6.pin(),
            pin_d5: pin_d5.pin(),
            pin_d4: pin_d4.pin(),
            pin_d3: pin_d3.pin(),
            pin_d2: pin_d2.pin(),
            pin_d1: pin_d1.pin(),
            pin_d0: pin_d0.pin(),
            pin_vsync: pin_vsync.pin(),
            pin_href: pin_href.pin(),
            pin_pclk: pin_pclk.pin(),

            xclk_freq_hz: xclk_freq_hz.0 as i32,
            ledc_timer,
            ledc_channel,

            pixel_format,
            frame_size,

            jpeg_quality,
            fb_count,
            fb_location,
            grab_mode,

            __bindgen_anon_1: camera::camera_config_t__bindgen_ty_1 {
                pin_sccb_sda: pin_sda.pin(),
            },
            __bindgen_anon_2: camera::camera_config_t__bindgen_ty_2 {
                pin_sccb_scl: pin_scl.pin(),
            },

            sccb_i2c_port: -1,
        };

        esp!(unsafe { camera::esp_camera_init(&cam_config) })?;

        Ok(Self { _p: PhantomData })
    }
}

impl<'a> Drop for Camera<'a> {
    fn drop(&mut self) {
        info!("Dropping Camera");
        if esp!(unsafe { camera::esp_camera_deinit() }).is_err() {
            error!("Error dropping Camera!");
        }
    }
}

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

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
        framesize_t_FRAMESIZE_QVGA,
        20,
        1,
        camera_fb_location_t_CAMERA_FB_IN_PSRAM,
        camera_grab_mode_t_CAMERA_GRAB_WHEN_EMPTY,
    )
    .unwrap();
    info!("Camera instance created");

    drop(camera);

    loop {
        FreeRtos::delay_ms(500u32);
    }
}
