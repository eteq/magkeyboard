#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Flex, Level, Output, OutputDrive, Pull};
use embassy_nrf::pwm::DutyCycle;
use embassy_nrf::{bind_interrupts, peripherals, pwm, saadc, spim, twim, uarte};
use embassy_nrf::timer::Frequency;
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

#[allow(dead_code)]
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

const SCHMIDT_THRESHOLD: f32 = 0.2f32;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut nrf_config = embassy_nrf::config::Config::default();
    nrf_config.hfclk_source = embassy_nrf::config::HfclkSource::ExternalXtal;
    nrf_config.lfclk_source = embassy_nrf::config::LfclkSource::ExternalXtal;
    //nrf_config.dcdc = embassy_nrf::config::DcdcConfig {reg0: false, reg1:true, reg0_voltage: None}; // TODO: decide if dc/dc is worth it in vusb or battery mode
    let mut p = embassy_nrf::init(nrf_config);

    // setup UART for comms
    // let mut uarte_config = uarte::Config::default();
    // uarte_config.parity = uarte::Parity::EXCLUDED;
    // uarte_config.baudrate = uarte::Baudrate::BAUD115200;

    // let mut uart = uarte::Uarte::new(p.UARTE0, p.P1_05, p.P1_03, Irqs, uarte_config);

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

    let mut adc = saadc::Saadc::new(p.SAADC, Irqs, saadc_config, channel_configs);
    adc.calibrate().await;

    //green LED on to indicate setup complete
    pwm.set_all_duties([
        DutyCycle::normal(0),
        DutyCycle::normal(255),
        DutyCycle::normal(0),
        DutyCycle::normal(0),
    ]);

    // select key 2 on each channel
    mux_a.set_low();
    mux_b.set_high();
    Timer::after(Duration::from_millis(10)).await;

    // enable all the muxes 
    for mux in muxens.iter_mut() {
        mux.set_low();
    }
    Timer::after(Duration::from_millis(100)).await; // let things settle


    defmt::info!("starting loop");

    
    let mut leds = [smart_leds::RGB8::new(0, 0, 0); N_KEYS];
    let mut smoothed_val: f32 = 2300.;
    let mut keyval: f32 = 0.5;
    let mut keymin = 2250f32;
    let mut keymax = 2310f32; 
    let mut min_max = keymax;
    let mut max_min = keymin;
    let mut pressed: bool = false;
    let mut sv2: f32 = 2000.;
    loop {
        let mut bufs = [[[0; 6]; 100]; 2];

        adc
        .run_task_sampler(
            p.TIMER0.reborrow(),
            p.PPI_CH0.reborrow(),
            p.PPI_CH1.reborrow(),
            Frequency::F1MHz,
            100, // gives us 10KHz
            &mut bufs,
            move |_buf| { 
                saadc::CallbackResult::Stop 
            },
        )
        .await;

        for samplei in 0..bufs[0].len() {
            smoothed_val = update_smooth(bufs[0][samplei][3], smoothed_val);
            keyval = update_keyval(smoothed_val, &mut keymin, &mut keymax, &mut min_max, &mut max_min);
            //defmt::info!("k,s,mi,mx:[{},{},{},{}]", keyval, smoothed_val, keymin, keymax);
            sv2 = update_smooth(bufs[0][samplei][1], sv2);
        }
        defmt::info!("k,s,mi,mx,sc2:[{},{},{},{},{}]", keyval, smoothed_val, keymin, keymax, sv2);
        if pressed {
            if keyval > 0.5+SCHMIDT_THRESHOLD {
                pressed = false;
            }
        } else {
            if keyval < 0.5-SCHMIDT_THRESHOLD {
                pressed = true;
            }
        }

        let rgb: Srgb = Hsv::new(keyval*270f32, 1f32, 0.1f32).into_color();
        let rgb: Srgb<u8> = rgb.into_format();
        leds[14] = smart_leds::RGB8::new(rgb.red, rgb.green, rgb.blue);
        keyleds
            .write(leds.clone())
            .expect("couldn't turn on key leds");

        if pressed {
            // cyan
            pwm.set_all_duties([
                DutyCycle::normal(100),
                DutyCycle::normal(50),
                DutyCycle::normal(0),
                DutyCycle::normal(0),
            ]);
        } else {
            // orange
            pwm.set_all_duties([
                DutyCycle::normal(0),
                DutyCycle::normal(100),
                DutyCycle::normal(100),
                DutyCycle::normal(0),
            ]);
        }
    }
}

const ALPHA: f32 = 0.05f32;
fn update_smooth(adc_reading: i16, previous: f32) -> f32 {
    (1f32-ALPHA)*previous + ALPHA*(adc_reading as f32)
}
const RELAXATION_RATE: f32 = 0.001f32;
const RELAXATION_MAX_FRAC: f32 = 0.2f32;
fn update_keyval(smoothed_val: f32, min_val: &mut f32, max_val: &mut f32, min_max: &mut f32, max_min: &mut f32) -> f32 {
    let range = *max_val - *min_val;
    if smoothed_val < *min_val {
        *min_val = smoothed_val;
        *max_min = smoothed_val + range * RELAXATION_MAX_FRAC;
    } else if *min_val < *max_min {
        *min_val += RELAXATION_RATE;
    }
    if smoothed_val > *max_val {
        *max_val = smoothed_val;
        *min_max = smoothed_val - range * RELAXATION_MAX_FRAC;
    } else if *max_val > *min_max {
        *max_val -= RELAXATION_RATE;
    }

    let result = (smoothed_val - *min_val) / range;
    if result < 0.0 {
        0.0
    } else if result > 1.0 {
        1.0
    } else {
        result
    }

}
