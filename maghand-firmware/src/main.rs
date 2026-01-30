#![no_std]
#![no_main]

use embedded_hal::digital::OutputPin;

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Flex, Level, Output, OutputDrive, Pull};
use embassy_nrf::pwm::DutyCycle;
use embassy_nrf::{Peri, bind_interrupts, peripherals, pwm, saadc, spim, twim, uarte, usb};
use embassy_time::Timer;
use embassy_nrf::timer::Frequency;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::lazy_lock::LazyLock;
use embassy_sync::channel::Channel;
use embassy_sync::pubsub::{PubSubChannel, Subscriber};
use embassy_nrf::usb::Driver;
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use {defmt_rtt as _, panic_probe as _};

use static_cell::ConstStaticCell;

use heapless::index_map::FnvIndexMap;

use smart_leds::SmartLedsWrite;
use ws2812_spi::Ws2812;

mod hardware_consts;
use hardware_consts::*;
mod keys;
mod usb_kb;


// alloc only needed for lsm6ds3tr crate.  Might not be worth that.
use lsm6ds3tr;
use embedded_alloc::LlffHeap as Heap;
#[global_allocator]
static HEAP: Heap = Heap::empty();

// static sync structures
//const N_CHANNEL_BUFFER: usize = 32;
//static CHANNEL: Channel<ThreadModeRawMutex, keys::KeySignal, N_CHANNEL_BUFFER> = Channel::new();
const KEYCHANGE_BUS_CAP: usize = 32;
const KEYCHANGE_BUS_SUBS: usize = 5;
static KEYCHANGE_BUS: PubSubChannel<ThreadModeRawMutex, keys::KeySignal, KEYCHANGE_BUS_CAP, KEYCHANGE_BUS_SUBS, N_KEYS> = PubSubChannel::new();

static KEYS_MUTEX_LAZY: LazyLock<Mutex<ThreadModeRawMutex, [keys::AnalogKey<ThreadModeRawMutex>; N_KEYS]>> = LazyLock::new(
    || Mutex::new(
        core::array::from_fn(|i| keys::AnalogKey::new(KEY_NAMES[i], 
            Some(KEYCHANGE_BUS.publisher().expect("couldn't make another keychange publisher"))))
    )
);

static KEY_INDEX_MAP: LazyLock<FnvIndexMap<u8, usize, 32>> = LazyLock::new(|| {
    let mut keys_to_index: FnvIndexMap<u8, usize, 32> = Default::default();
    for (i, knm) in KEY_NAMES.iter().enumerate() {
        keys_to_index.insert(*knm, i).unwrap();
    }
    keys_to_index
});


bind_interrupts!(struct Irqs {
    SAADC => saadc::InterruptHandler;
    UARTE0 => uarte::InterruptHandler<peripherals::UARTE0>;
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
    USBD => usb::InterruptHandler<peripherals::USBD>;
    CLOCK_POWER => usb::vbus_detect::InterruptHandler;
});

fn vhi_on(vhi_pin: &mut Flex) {
    vhi_pin.set_low();
    vhi_pin.set_as_output(OutputDrive::Standard);
}

fn vhi_off(vhi_pin: &mut Flex) {
    vhi_pin.set_as_input(Pull::None);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut nrf_config = embassy_nrf::config::Config::default();
    nrf_config.hfclk_source = embassy_nrf::config::HfclkSource::ExternalXtal;
    nrf_config.lfclk_source = embassy_nrf::config::LfclkSource::ExternalXtal;
    //nrf_config.dcdc = embassy_nrf::config::DcdcConfig {reg0: false, reg1:true, reg0_voltage: None}; // TODO: decide if dc/dc is worth it in vusb or battery mode
    let mut p = embassy_nrf::init(nrf_config);


    // setup vhi gpio
    let mut vhi_pin = Flex::new(p.P1_12);
    vhi_off(&mut vhi_pin); // should be the default state, but just in case

    // setup board LEDs using PWM - note these are active *low*, so idle level is high
    let mut pwm_config = embassy_nrf::pwm::SimpleConfig::default();
    // not much visible difference between standard and high drive on the board LEDs... but high drive seems to give *less* power usage...
    pwm_config.ch0_drive = OutputDrive::HighDrive;
    pwm_config.ch1_drive = OutputDrive::HighDrive;
    pwm_config.ch2_drive = OutputDrive::HighDrive;
    pwm_config.ch0_idle_level = Level::High;
    pwm_config.ch1_idle_level = Level::High;
    pwm_config.ch2_idle_level = Level::High;
    pwm_config.prescaler = pwm::Prescaler::Div16; // 1 Mhz - TODO: see if if this has any significant power/performance impact
    pwm_config.max_duty = 255; // compatible with u8 rgb values

    let mut pwm = pwm::SimplePwm::new_3ch(p.PWM0, p.P0_26, p.P0_30, p.P0_06, &pwm_config);
    pwm.set_all_duties([
        DutyCycle::normal(255),
        DutyCycle::normal(0),
        DutyCycle::normal(0),
        DutyCycle::normal(0),
    ]); //only red on
    pwm.enable();

    // setup DIN for key leds - note only works if vhi is on
    let mut spi_config = spim::Config::default();
    spi_config.frequency = spim::Frequency::M4;
    let mut keyleds = Ws2812::new(spim::Spim::new_txonly_nosck(
        p.SPI3, Irqs, p.P0_15, spi_config,
    ));
    // power up vhi for the leds
    vhi_on(&mut vhi_pin);
    Timer::after(LED_POWERUP_TIME).await;
    // blink them for a moment at startup
    keyleds
        .write([smart_leds::RGB8::new(1, 0, 1); N_KEYS].iter().cloned())
        .expect("couldn't blink key leds");
    Timer::after_millis(50).await;
    keyleds
        .write([smart_leds::RGB8::new(0, 0, 0); N_KEYS].iter().cloned())
        .expect("couldn't clear key leds");
    //TODO: compare to using pwm for key leds p.P0_15

    // setup imu on internal i2c
    // total power usage with both the imu and the uart output is ~0.7 mA
    // 6D_PWR: p.P1_08 -> output high to enable imu power
    // 6D_INT1: p.P0_11 -> interrupt input from imu
    // 6D_i2C_SDA: p.P0_07
    // 6D_i2C_SCL: p.P0_27
    let mut imu_pwr_pin = Output::new(p.P1_08, Level::Low, OutputDrive::Standard);
    imu_pwr_pin.set_high();
    Timer::after(IMU_POWERUP_TIME).await;

    let mut twim_config = twim::Config::default();
    twim_config.frequency = twim::Frequency::K100;
    twim_config.sda_pullup = false;
    twim_config.scl_pullup = false;
    static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]); // not sure what size is needed for lsm6ds3tr crate, this is just a guess
    let i2c = twim::Twim::new(
        p.TWISPI0,
        Irqs,
        p.P0_07,
        p.P0_27,
        twim_config,
        RAM_BUFFER.take(),
    );

    // TODO: fix lsm6ds3tr to allow gyro setting - a register should be pub that isnt: https://gitlab.com/mtczekajlo/lsm6ds3tr-rs/-/merge_requests/7
    let accel_settings = lsm6ds3tr::AccelSettings::default()
        .with_sample_rate(lsm6ds3tr::AccelSampleRate::_208Hz)
        .with_scale(lsm6ds3tr::AccelScale::_2G);
    let imu_settings = lsm6ds3tr::LsmSettings::basic()
        .with_low_performance_mode()
        .with_accel(accel_settings);
    let mut imu = lsm6ds3tr::LSM6DS3TR::new(lsm6ds3tr::interface::I2cInterface::new(i2c))
        .with_settings(imu_settings);
    imu.init().expect("LSM6DS3TR-C initialization failure!");

    // setup GPIO to enable various keys
    let mut muxen01 = Output::new(p.P1_13, Level::High, OutputDrive::Standard);
    let mut muxen23 = Output::new(p.P1_14, Level::High, OutputDrive::Standard);
    let mut muxen45 = Output::new(p.P1_15, Level::High, OutputDrive::Standard);
    let mut muxens = [&mut muxen01, &mut muxen23, &mut muxen45];

    let mut mux_a = Output::new(p.P0_09, Level::Low, OutputDrive::Standard);
    let mut mux_b = Output::new(p.P0_10, Level::Low, OutputDrive::Standard);

    // setup ADC for key position reading
    let keyset0_channel_config = saadc::ChannelConfig::single_ended(p.P0_02.reborrow()); // 01X / 0
    let keyset1_channel_config = saadc::ChannelConfig::single_ended(p.P0_03.reborrow()); // 01Y / 1
    let keyset2_channel_config = saadc::ChannelConfig::single_ended(p.P0_04.reborrow()); // 23X / 2
    let keyset3_channel_config = saadc::ChannelConfig::single_ended(p.P0_05.reborrow()); // 23Y / 3
    let keyset4_channel_config = saadc::ChannelConfig::single_ended(p.P0_28.reborrow()); // 45X / 4
    let keyset5_channel_config = saadc::ChannelConfig::single_ended(p.P0_29.reborrow()); // 45Y / 5
    let mut channel_configs = [
        keyset0_channel_config,
        keyset1_channel_config,
        keyset2_channel_config,
        keyset3_channel_config,
        keyset4_channel_config,
        keyset5_channel_config,
    ];
    for config in channel_configs.iter_mut() {
        config.reference = saadc::Reference::VDD1_4;
        config.gain = saadc::Gain::GAIN1_4;
        config.resistor = saadc::Resistor::BYPASS;
        config.time = saadc::Time::_10US;
    }

    let mut saadc_config = saadc::Config::default();
    saadc_config.resolution = saadc::Resolution::_12BIT;
    saadc_config.oversample = saadc::Oversample::BYPASS;

    let adc = saadc::Saadc::new(p.SAADC, Irqs, saadc_config, channel_configs);
    adc.calibrate().await;


    // select key 2 on each channel
    mux_a.set_low();
    mux_b.set_high();
    Timer::after_millis(10).await;

    // enable all the muxes 
    for mux in muxens.iter_mut() {
        mux.set_low();
    }
    Timer::after_millis(100).await; // let things settle

    // set up USB
    let usb_driver = Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs));

    spawner.spawn(usb_kb::usb_task(usb_driver, 
        KEYCHANGE_BUS.subscriber().expect("couldn't make usb keychange subscriber"))
    ).expect("failed to spawn USB task");


    //green LED on to indicate setup complete
    pwm.set_all_duties([
        DutyCycle::normal(0),
        DutyCycle::normal(100),
        DutyCycle::normal(0),
        DutyCycle::normal(0),
    ]);

    defmt::debug!("starting adc and main loop");

    spawner.spawn(adc_sampler(adc, 
                              p.TIMER0, 
                              p.PPI_CH0,
                              p.PPI_CH1,
                              mux_a,
                              mux_b)).expect("failed to spawn adc sampler");


    loop {
        // just wait, all the action should happen in usb
        Timer::after_millis(500).await;
    }
}

// 12.5 usec (10+2.5) is 80 kHz sample rate
// sample rate is then 80 kHz / 6 channels / 4 mux settings / NSAMP = 5 msec for NSAMP=25
const NCHAN: usize = 6;
const NSAMP: usize = 256;
#[embassy_executor::task]
async fn adc_sampler(mut adc: saadc::Saadc<'static, NCHAN>, 
                     mut timer: Peri<'static, peripherals::TIMER0>, 
                     mut ppi1: Peri<'static, peripherals::PPI_CH0>, 
                     mut ppi2: Peri<'static, peripherals::PPI_CH1>,
                     mut mux_a: Output<'static>,
                     mut mux_b: Output<'static>,) {

    let keys_mutex= KEYS_MUTEX_LAZY.get();
    
    let mut bufs = [[[0; NCHAN]; NSAMP]; 2];
    let bufs_inner_size = bufs[0].len();

    #[cfg(feature = "adc_debug")]
    let mut debug_key_index: usize = 0;

    let key_index_map = KEY_INDEX_MAP.get();

    loop {
        for muxsetting in keys::MuxSpec::iterator() {
            mux_a.set_level(muxsetting.a);
            mux_b.set_level(muxsetting.b);
            if let Some(settle_time) = MUX_SETTLE_TIME {
                Timer::after(settle_time).await;
            }

            #[cfg(feature = "adc_debug")]
            let adcstart = Instant::now();

            adc
                .run_task_sampler(
                    timer.reborrow(),
                    ppi1.reborrow(),
                    ppi2.reborrow(),
                    Frequency::F8MHz,
                    100, 
                    &mut bufs,
                    move |buf| {
                        if buf.len() !=  bufs_inner_size {
                            defmt::warn!("adc buffer size mismatch: {} != {}", buf.len(), bufs_inner_size);
                        }
                        saadc::CallbackResult::Stop
                    },
                ).await;

             #[cfg(feature = "adc_debug")]
            let adcend = Instant::now();

            let data = bufs[0];
            {  // scope for locking mutex
                let mut keys = keys_mutex.lock().await;

                for chan in 0..NCHAN {
                    let keyname = (chan*10) as u8 + muxsetting.index();
                    match key_index_map.get(&keyname) {
                        Some(&keyindex) => {
                            for samp in data.iter() {
                                keys[keyindex].update_value_adc((*samp)[chan]);
                            }
                            #[cfg(feature = "adc_debug")]
                            if keyindex == debug_key_index {
                                let mut values = [0i16; NSAMP];
                                for (i, samp) in data.iter().enumerate() {
                                    values[i] = (*samp)[chan];
                                }
                                defmt::debug!("Key: {}; adctime us: {},{}; values: {}", 
                                              keyname, adcstart.as_micros(), adcend.as_micros(), values);
                            }
                        }
                        None => {
                            defmt::trace!("No key found for key name {}", keyname);
                        }
                    }
                }
            }
        }


        #[cfg(feature = "adc_debug")]
        { debug_key_index = (debug_key_index + 1) % N_KEYS; }

    }
}