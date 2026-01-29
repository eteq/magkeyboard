use core::sync::atomic::{AtomicBool, Ordering};

use embassy_nrf::usb::Driver;
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_usb::{Builder, Handler};
use embassy_usb::class::hid::{RequestHandler, State, HidReaderWriter, ReportId};
use embassy_usb::control::OutResponse;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::signal::Signal;
use embassy_sync::channel::Receiver;
use embassy_futures::select::{select, Either};
use embassy_futures::join::join;
use crate::keys::KeySignal;
use crate::N_CHANNEL_BUFFER;

const READ_REPORT_MAX_SIZE: usize = 1;
const WRITE_REPORT_MAX_SIZE: usize = 8;

static SUSPENDED: AtomicBool = AtomicBool::new(false);

#[embassy_executor::task]
//pub async fn usb_task(usbd: Peri<'static, USBD>) {
pub async fn usb_task(driver: Driver<'static, HardwareVbusDetect>, key_receiver:Receiver<'static, ThreadModeRawMutex, KeySignal, N_CHANNEL_BUFFER>) {
    
    let mut config = embassy_usb::Config::new(0xc0de, 0x1983);
    config.manufacturer = Some("Erik's Not-Industries");
    config.product = Some("maghand-keyboard");
    config.serial_number = Some("12345678");
    config.max_power = 500;
    config.max_packet_size_0 = 64;
    config.supports_remote_wakeup = true;
    // config.composite_with_iads = false;
    // config.device_class = 0;
    // config.device_sub_class = 0;
    // config.device_protocol = 0;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut request_handler = MaghandRequestHandler {};
    let mut device_handler = MaghandDeviceHandler::new();

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );    

    builder.handler(&mut device_handler);
    
    // Create classes on the builder.
    let config = embassy_usb::class::hid::Config {
        report_descriptor: KeyboardReport::desc(),
        request_handler: None,
        poll_ms: 60,
        max_packet_size: 64,
    };
    let hid = HidReaderWriter::<_, READ_REPORT_MAX_SIZE, WRITE_REPORT_MAX_SIZE>::new(&mut builder, &mut state, config);

    // Build the builder.
    let mut usb = builder.build();

    let remote_wakeup: Signal<CriticalSectionRawMutex, _> = Signal::new();

    // Run the USB device.
    let usb_fut = async {
        loop {
            usb.run_until_suspend().await;
            match select(usb.wait_resume(), remote_wakeup.wait()).await {
                Either::First(_) => (),
                Either::Second(_) =>  {
                    match usb.remote_wakeup().await {
                        Ok(()) => defmt::info!("USB Remote wakeup successful"),
                        Err(e) => defmt::info!("USB Remote wakeup failed: {:?}", e),
                    };
                },
            }
        }
    };

    let (reader, mut writer) = hid.split();

    // Do stuff with the class!
    let in_fut = async {
        loop {
            let toggle_data = key_receiver.receive().await;
            defmt::info!("toggled key {} to {}", toggle_data.keynumber, toggle_data.toggle_on);

            if SUSPENDED.load(Ordering::Acquire) {
                defmt::info!("Triggering remote wakeup");
                remote_wakeup.signal(());
            } 
            // else {
            //     let report = KeyboardReport {
            //         keycodes: [4, 0, 0, 0, 0, 0],
            //         leds: 0,
            //         modifier: 0,
            //         reserved: 0,
            //     };
            //     match writer.write_serialize(&report).await {
            //         Ok(()) => {}
            //         Err(e) => defmt::warn!("Failed to send report: {:?}", e),
            //     };
            // }
        }
    };

    let out_fut = async {
        reader.run(false, &mut request_handler).await;
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, join(in_fut, out_fut)).await;
}


struct MaghandRequestHandler {}

impl RequestHandler for MaghandRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        defmt::info!("Get report for {:?}", id);
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        defmt::info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, dur: u32) {
        defmt::info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        defmt::info!("Get idle rate for {:?}", id);
        None
    }
}

struct MaghandDeviceHandler {
    configured: AtomicBool,
}

impl MaghandDeviceHandler {
    fn new() -> Self {
        MaghandDeviceHandler {
            configured: AtomicBool::new(false),
        }
    }
}

impl Handler for MaghandDeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        SUSPENDED.store(false, Ordering::Release);
        if enabled {
            defmt::info!("Device enabled");
        } else {
            defmt::info!("Device disabled");
        }
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        defmt::info!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        defmt::info!("USB address set to: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        if configured {
            defmt::info!("Device configured, it may now draw up to the configured current limit from Vbus.")
        } else {
            defmt::info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }

    fn suspended(&mut self, suspended: bool) {
        if suspended {
            defmt::info!("Device suspended, the Vbus current limit is 500µA (or 2.5mA for high-power devices with remote wakeup enabled).");
            SUSPENDED.store(true, Ordering::Release);
        } else {
            SUSPENDED.store(false, Ordering::Release);
            if self.configured.load(Ordering::Relaxed) {
                defmt::info!("Device resumed, it may now draw up to the configured current limit from Vbus");
            } else {
                defmt::info!("Device resumed, the Vbus current limit is 100mA");
            }
        }
    }
}