use std::marker::PhantomData;

use esp_idf_svc::sys::{c_camera, esp};
use esp_idf_svc::{
    hal::{
        gpio::{self, Pin},
        units::Hertz,
    },
    sys::c_camera::gpio_num_t_GPIO_NUM_NC,
    sys::EspError,
};
use log::{error, info};

pub struct FrameBuffer<'a> {
    fb: *mut c_camera::camera_fb_t,
    _p: PhantomData<&'a c_camera::camera_fb_t>,
}

impl<'a> FrameBuffer<'a> {
    pub fn data(&self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts((*self.fb).buf, (*self.fb).len) }
    }

    pub fn fb_return(&self) {
        unsafe { c_camera::esp_camera_fb_return(self.fb) }
    }
}

impl Drop for FrameBuffer<'_> {
    fn drop(&mut self) {
        self.fb_return();
    }
}

pub struct Camera<'a> {
    _p: PhantomData<&'a ()>,
}

impl Camera<'_> {
    #[allow(clippy::too_many_arguments)]
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
        ledc_timer: c_camera::ledc_timer_t,
        ledc_channel: c_camera::ledc_channel_t,
        pixel_format: c_camera::pixformat_t,
        frame_size: c_camera::framesize_t,
        jpeg_quality: i32,
        fb_count: usize,
        fb_location: c_camera::camera_fb_location_t,
        grab_mode: c_camera::camera_grab_mode_t,
    ) -> Result<Self, EspError> {
        let cam_config = c_camera::camera_config_t {
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

            __bindgen_anon_1: c_camera::camera_config_t__bindgen_ty_1 {
                pin_sccb_sda: pin_sda.pin(),
            },
            __bindgen_anon_2: c_camera::camera_config_t__bindgen_ty_2 {
                pin_sccb_scl: pin_scl.pin(),
            },

            sccb_i2c_port: -1,
        };

        esp!(unsafe { c_camera::esp_camera_init(&cam_config) })?;

        Ok(Self { _p: PhantomData })
    }

    pub fn get_framebuffer(&self) -> Option<FrameBuffer> {
        let fb = unsafe { c_camera::esp_camera_fb_get() };
        if fb.is_null() {
            None
        } else {
            Some(FrameBuffer {
                fb,
                _p: PhantomData,
            })
        }
    }
}

impl Drop for Camera<'_> {
    fn drop(&mut self) {
        info!("Dropping Camera");
        if esp!(unsafe { c_camera::esp_camera_deinit() }).is_err() {
            error!("Error dropping Camera!");
        }
    }
}
