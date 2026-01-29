use core::future::Future;
use embassy_time::Timer;
use embassy_nrf::usb::Driver;
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_usb::Builder;
use embassy_usb::class::hid::{self, RequestHandler, State, HidWriter};
use usbd_hid::descriptor::{MouseReport, SerializedDescriptor};


const WRITE_REPORT_MAX_SIZE: usize = 5;

pub struct USBKeyboard<'a> {
    config_descriptor: [u8; 256],
    bos_descriptor: [u8; 256],
    msos_descriptor: [u8; 256],
    control_buf: [u8; 64],
    request_handler: Option<MaghandRequestHandler>,
    state: Option<State<'a>>,
    builder: Option<Builder<'a, Driver<'a, HardwareVbusDetect>>>,
    writer: Option<HidWriter<'a, Driver<'a, HardwareVbusDetect>, WRITE_REPORT_MAX_SIZE>>,
    usb: Option<embassy_usb::UsbDevice<'a, Driver<'a, HardwareVbusDetect>>>,
}

impl Default for USBKeyboard<'_> {
    fn default() -> Self {
        Self {
            config_descriptor: [0; 256],
            bos_descriptor: [0; 256],
            msos_descriptor: [0; 256],
            control_buf: [0; 64],
            request_handler: None,
            state: None,
            builder: None,
            writer: None,
            usb: None
        }
    }
}

impl<'a> USBKeyboard<'a> {
    pub fn setup(&mut self, driver: Driver<'a, HardwareVbusDetect>){

        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Erik's Not-Industries");
        config.product = Some("maghand-keyboard");
        config.serial_number = Some("12345678");
        config.max_power = 500;
        config.max_packet_size_0 = 64;
        config.composite_with_iads = false;
        config.device_class = 0;
        config.device_sub_class = 0;
        config.device_protocol = 0;

        self.request_handler = Some(MaghandRequestHandler {});

        self.state = Some(State::new());

        let builder: Builder<'a, Driver<'a, HardwareVbusDetect>> = Builder::new(
            driver,
            config,
            &mut self.config_descriptor,
            &mut self.bos_descriptor,
            &mut self.msos_descriptor,
            &mut self.control_buf,
        );

        let hid_config = hid::Config {
            report_descriptor: MouseReport::desc(),
            request_handler: Some(self.request_handler.as_mut().unwrap()),
            poll_ms: 60,
            max_packet_size: 8,
        };

        let writer = HidWriter::<_, WRITE_REPORT_MAX_SIZE>::new(self.builder.as_mut().unwrap(), 
            self.state.as_mut().unwrap(), 
            hid_config);
        self.writer = Some(writer);

        self.usb = Some(builder.build());

    }

    pub fn get_runners(&'a mut self) -> (impl Future, impl Future) {
        let usb_fut = self.usb.as_mut().unwrap().run();

        let hid_fut = hid_action(self.writer.as_mut().unwrap());

        (usb_fut, hid_fut)
    }
}


async fn hid_action<'a>(writer: &mut HidWriter<'a, Driver<'a, HardwareVbusDetect>, WRITE_REPORT_MAX_SIZE>) {
    let mut y: i8 = 5;
        loop {
            Timer::after_millis(500).await;

            y = -y;
            let report = MouseReport {
                buttons: 0,
                x: 0,
                y,
                wheel: 0,
                pan: 0,
            };
            match writer.write_serialize(&report).await {
                Ok(()) => {}
                Err(e) => defmt::warn!("Failed to send report: {:?}", e),
            }
        }
}


struct MaghandRequestHandler {}

impl RequestHandler for MaghandRequestHandler {
}