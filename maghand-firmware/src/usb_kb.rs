use embassy_futures::join::join;
use embassy_time::Timer;
use embassy_nrf::usb::Driver;
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_usb::Builder;
use embassy_usb::class::hid::{RequestHandler, State, HidWriter};
use usbd_hid::descriptor::{MouseReport, SerializedDescriptor};

const WRITE_REPORT_MAX_SIZE: usize = 5;

#[embassy_executor::task]
//pub async fn usb_task(usbd: Peri<'static, USBD>) {
pub async fn usb_task(driver: Driver<'static, HardwareVbusDetect>) {
    
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

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut request_handler = MaghandRequestHandler {};

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );    
    
    // Create classes on the builder.
    let config = embassy_usb::class::hid::Config {
        report_descriptor: MouseReport::desc(),
        request_handler: Some(&mut request_handler),
        poll_ms: 60,
        max_packet_size: 8,
    };

    let mut writer = HidWriter::<_, 5>::new(&mut builder, &mut state, config);

    // Build the builder.
    let mut usb = builder.build();

    let usb_fut = usb.run();

    // Do stuff with the class!
    let hid_fut = hid_action(&mut writer);

    join(usb_fut, hid_fut).await;
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