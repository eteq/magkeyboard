#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Flex, Level, Output, OutputDrive, Pull};
use embassy_nrf::pwm::DutyCycle;
use embassy_nrf::{bind_interrupts, peripherals, pwm, saadc, spim, twim, uarte,timer};
use embassy_time::{Duration, Timer};
use static_cell::ConstStaticCell;
use {defmt_rtt as _, panic_probe as _};

use palette::{Hsv, IntoColor, Srgb};

use smart_leds::SmartLedsWrite;
use ws2812_spi::Ws2812;


// alloc only needed for lsm6ds3tr crate.  Might not be worth that.
use lsm6ds3tr;
use embedded_alloc::LlffHeap as Heap;
#[global_allocator]
static HEAP: Heap = Heap::empty();


const N_KEYS: usize = 24;
const LED_POWERUP_TIME: Duration = Duration::from_millis(1); // this is just a guess - implicitly it's everything connected to vhi
const IMU_POWERUP_TIME: Duration = Duration::from_millis(35); // lsm6ds3tr datasheet

bind_interrupts!(struct Irqs {
    SAADC => saadc::InterruptHandler;
    UARTE0 => uarte::InterruptHandler<peripherals::UARTE0>;
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

fn vhi_on(vhi_pin: &mut Flex) {
    vhi_pin.set_low();
    vhi_pin.set_as_output(OutputDrive::Standard);
}

fn vhi_off(vhi_pin: &mut Flex) {
    vhi_pin.set_as_input(Pull::None);
}

#[allow(dead_code)]
fn hsv_on_board_leds(pwm: &mut pwm::SimplePwm, h: f32, s: f32, v: f32) {
    let rgb: Srgb = Hsv::new(h, s, v).into_color();
    let rgb: Srgb<u8> = rgb.into_format();

    let r_duty = DutyCycle::normal(rgb.red as u16);
    let g_duty = DutyCycle::normal(rgb.green as u16);
    let b_duty = DutyCycle::normal(rgb.blue as u16);
    pwm.set_all_duties([r_duty, g_duty, b_duty, DutyCycle::normal(0)]);
}

fn hsv_cycle_ws2812_data(
    phase: f32,
    s: f32,
    v: f32,
    fracrainbow: f32,
) -> [smart_leds::RGB8; N_KEYS] {
    let mut data = [smart_leds::RGB8::default(); N_KEYS];
    for (i, led) in data.iter_mut().enumerate() {
        let h = ((i as f32) / (N_KEYS as f32) * fracrainbow + phase) % 1.0 * 360.;
        let rgb: Srgb = Hsv::new(h, s, v).into_color();
        let rgb: Srgb<u8> = rgb.into_format();
        *led = smart_leds::RGB8::new(rgb.red, rgb.green, rgb.blue);
    }
    data
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut nrf_config = embassy_nrf::config::Config::default();
    nrf_config.hfclk_source = embassy_nrf::config::HfclkSource::ExternalXtal;
    nrf_config.lfclk_source = embassy_nrf::config::LfclkSource::ExternalXtal;
    //nrf_config.dcdc = embassy_nrf::config::DcdcConfig {reg0: false, reg1:true, reg0_voltage: None}; // TODO: decide if dc/dc is worth it in vusb or battery mode
    let mut p = embassy_nrf::init(nrf_config);

    // setup UART for comms
    let mut uarte_config = uarte::Config::default();
    uarte_config.parity = uarte::Parity::EXCLUDED;
    uarte_config.baudrate = uarte::Baudrate::BAUD115200;

    let mut uart = uarte::Uarte::new(p.UARTE0, p.P1_05, p.P1_03, Irqs, uarte_config);

    // setup vhi gpio
    let mut vhi_pin = Flex::new(p.P1_12);
    vhi_off(&mut vhi_pin); // should be the default state, but just in case

    // setup board LEDs using PWM - note these are active *low*, so idle level is high
    // power usage with no leds in is ~0.9 mA
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
    Timer::after(Duration::from_millis(100)).await;
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
    let keyset0_channel_config = saadc::ChannelConfig::single_ended(p.P0_02.reborrow());
    let keyset1_channel_config = saadc::ChannelConfig::single_ended(p.P0_03.reborrow());
    let keyset2_channel_config = saadc::ChannelConfig::single_ended(p.P0_04.reborrow());
    let keyset3_channel_config = saadc::ChannelConfig::single_ended(p.P0_05.reborrow());
    let keyset4_channel_config = saadc::ChannelConfig::single_ended(p.P0_28.reborrow());
    let keyset5_channel_config = saadc::ChannelConfig::single_ended(p.P0_29.reborrow());
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

    let mut adc = saadc::Saadc::new(p.SAADC, Irqs, saadc_config, channel_configs);
    adc.calibrate().await;

    //green LED on to indicate setup complete
    pwm.set_all_duties([
        DutyCycle::normal(0),
        DutyCycle::normal(255),
        DutyCycle::normal(0),
        DutyCycle::normal(0),
    ]);
    // 32 will then be what's going into channel 3.  Also 0,1 will be disconnected but then connected in the second scan, while 45 will stay disconnected
    muxen01.set_high();
    muxen23.set_low();
    muxen45.set_high();
    mux_a.set_low();
    mux_b.set_high();
    Timer::after(Duration::from_millis(10)).await;

    defmt::info!("running sampler continuous");
    let mut bufs = [[[0; 6]; 500]; 2];
    let mut i: isize = -1;
    adc
        .run_task_sampler(
            p.TIMER0.reborrow(),
            p.PPI_CH0.reborrow(),
            p.PPI_CH1.reborrow(),
            timer::Frequency::F1MHz,
            1000, // this yields 1 khz
            &mut bufs,
            |_buf| {
                muxen01.set_low();
                i+=1;
                if i >= 1 { saadc::CallbackResult::Stop } else { saadc::CallbackResult::Continue }
            },
        )
        .await;

    defmt::info!("rinished sampler continuous");
    defmt::info!("bufs:{:?}", &bufs);
    loop {
        // main loop does nothing but blinky now
        Timer::after(Duration::from_millis(500)).await;
          pwm.set_all_duties([
                DutyCycle::normal(0),
                DutyCycle::normal(100),
                DutyCycle::normal(100),
                DutyCycle::normal(0),
            ]);
        Timer::after(Duration::from_millis(500)).await;
          pwm.set_all_duties([
                DutyCycle::normal(0),
                DutyCycle::normal(0),
                DutyCycle::normal(0),
                DutyCycle::normal(0),
            ]);
    defmt::info!("looped continuous");
    }
}
