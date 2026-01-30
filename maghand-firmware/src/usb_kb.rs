use crate::keys::{KeySignal, Layer, KEYMAP};
use crate::{KEYCHANGE_BUS_CAP, KEYCHANGE_BUS_SUBS};
use crate::hardware_consts::N_KEYS;
const N_KEYS_POWEROF2: usize = N_KEYS.next_power_of_two();

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_nrf::usb::Driver;
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::signal::Signal;
use embassy_sync::pubsub::{Subscriber, WaitResult};
use embassy_futures::select::{select, Either};
use embassy_futures::join::join;
use embassy_time::Timer;

use embassy_usb::{Builder, Handler};
use embassy_usb::class::hid::{RequestHandler, State, HidReaderWriter, ReportId};
use embassy_usb::control::OutResponse;

use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor, KeyboardUsage};

use heapless::index_set::FnvIndexSet;

const READ_REPORT_SIZE: usize = 1;
const WRITE_REPORT_SIZE: usize = 8;

static SUSPENDED: AtomicBool = AtomicBool::new(false);

#[embassy_executor::task]
pub async fn usb_task(driver: Driver<'static, HardwareVbusDetect>, 
                      mut key_subscriber: Subscriber<'static, ThreadModeRawMutex, KeySignal, KEYCHANGE_BUS_CAP, KEYCHANGE_BUS_SUBS, N_KEYS>) {
    
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
    let hid = HidReaderWriter::<_, READ_REPORT_SIZE, WRITE_REPORT_SIZE>::new(&mut builder, &mut state, config);

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

    //let key_index_map = KEY_INDEX_MAP.get();
    let keymap = KEYMAP.get();
    let mut keysdown: FnvIndexSet<u8, N_KEYS_POWEROF2> = FnvIndexSet::new(); //TODO: add some check that this is the next power-of-two greater than N_KEYS

    // this is where the signal comes in and the key press is sent
    let in_fut = async {
        loop {
            let toggle_data =  match key_subscriber.next_message().await {
                WaitResult::Lagged(n) => {
                    defmt::warn!("Key change subscriber lagged by {}", n);
                    continue;
                },
                WaitResult::Message(data) => { data }
            };
            defmt::debug!("toggled key {} to {}", toggle_data.keynumber, toggle_data.toggle_on);

            
            if SUSPENDED.load(Ordering::Acquire) {
                defmt::info!("Triggering remote wakeup");
                remote_wakeup.signal(());
                while SUSPENDED.load(Ordering::Acquire) {
                    //TODO: test the right delay time here
                    Timer::after(embassy_time::Duration::from_millis(5)).await;
                }
            }

            match keymap.get(&(toggle_data.keynumber, Layer::Default)) {
                Some(keycoderef) => {
                    let keycode = (*keycoderef) as u8;

                    if toggle_data.toggle_on {
                        keysdown.insert(keycode).expect("keysdown full");
                    } else {
                        let was_present = keysdown.remove(&keycode);
                        if !was_present {
                            defmt::warn!("On keyup, {} wasnt down", keycode);
                        }
                    }

                    let keycodes = {
                        if keysdown.len() > 6 {
                            [KeyboardUsage::KeyboardErrorRollOver as u8 ; 6]
                        } else {
                            let mut arr = [0u8; 6];
                            for (i, kc) in keysdown.iter().enumerate() {
                                arr[i] = *kc;
                            }
                            arr
                        }
                    };

                    let report = KeyboardReport {
                        keycodes: keycodes,
                        leds: 0,
                        modifier: 0,
                        reserved: 0,
                    };

                    defmt::debug!("Sending usb kb keycodes: {}", keycodes);

                    match writer.write_serialize(&report).await {
                        Ok(()) => {}
                        Err(e) => defmt::warn!("Failed to send report: {:?}", e),
                    };
                },
                None => { defmt::warn!("No keycode mapped for keynumber {}, skipping", toggle_data.keynumber);  },
            }
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
        defmt::debug!("Get report for {:?}", id);
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        defmt::debug!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, dur: u32) {
        defmt::debug!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        defmt::debug!("Get idle rate for {:?}", id);
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
            defmt::debug!("Device enabled");
        } else {
            defmt::debug!("Device disabled");
        }
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        defmt::debug!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        defmt::debug!("USB address set to: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        if configured {
            defmt::debug!("Device configured, it may now draw up to the configured current limit from Vbus.")
        } else {
            defmt::debug!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }

    fn suspended(&mut self, suspended: bool) {
        if suspended {
            defmt::debug!("Device suspended, the Vbus current limit is 500µA (or 2.5mA for high-power devices with remote wakeup enabled).");
            SUSPENDED.store(true, Ordering::Release);
        } else {
            SUSPENDED.store(false, Ordering::Release);
            if self.configured.load(Ordering::Relaxed) {
                defmt::debug!("Device resumed, it may now draw up to the configured current limit from Vbus");
            } else {
                defmt::debug!("Device resumed, the Vbus current limit is 100mA");
            }
        }
    }
}