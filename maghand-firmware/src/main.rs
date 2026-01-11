#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive, Flex, Pull};
use embassy_nrf::{bind_interrupts, saadc, peripherals, uarte};
use embassy_nrf::pwm::{Prescaler, SimplePwm, DutyCycle};
use embassy_time::Instant;
use {defmt_rtt as _, panic_probe as _};

use heapless::format;
use palette::{Hsv, Srgb, IntoColor};

bind_interrupts!(struct Irqs {
    SAADC => saadc::InterruptHandler;
    UARTE0 => uarte::InterruptHandler<peripherals::UARTE0>;
}); 

fn vhi_on(vhi_pin: &mut Flex) {
    vhi_pin.set_low();
    vhi_pin.set_as_output(OutputDrive::Standard);
}

fn vhi_off(vhi_pin: &mut Flex) {
    vhi_pin.set_as_input(Pull::None);
}

fn hsv_on_board_leds(pwm: &mut SimplePwm, h: f32, s: f32, v: f32) {
    let rgb: Srgb = Hsv::new(h, s, v).into_color();
    let rgb: Srgb<u8> = rgb.into_format();
    
    let r_duty = DutyCycle::normal(rgb.red as u16);
    let g_duty = DutyCycle::normal(rgb.green as u16);
    let b_duty = DutyCycle::normal(rgb.blue as u16);
    pwm.set_all_duties([r_duty, g_duty, b_duty, DutyCycle::normal(0)]);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut nrf_config = embassy_nrf::config::Config::default();
    nrf_config.hfclk_source = embassy_nrf::config::HfclkSource::ExternalXtal;
    nrf_config.lfclk_source = embassy_nrf::config::LfclkSource::ExternalXtal;
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
    let mut pwm_config = embassy_nrf::pwm::SimpleConfig::default();
    pwm_config.ch0_drive = OutputDrive::HighDrive;
    pwm_config.ch1_drive = OutputDrive::HighDrive;
    pwm_config.ch2_drive = OutputDrive::HighDrive;
    pwm_config.ch0_idle_level = Level::High;
    pwm_config.ch1_idle_level = Level::High;
    pwm_config.ch2_idle_level = Level::High;
    pwm_config.prescaler = Prescaler::Div16; // 1 Mhz - TODO: see if if this has any significant power/performance impact
    pwm_config.max_duty = 255; // compatible with u8 rgb values

    let mut pwm = SimplePwm::new_3ch(p.PWM0, p.P0_26, p.P0_30, p.P0_06, &pwm_config);
    pwm.set_all_duties([DutyCycle::normal(255), DutyCycle::normal(0), DutyCycle::normal(0), DutyCycle::normal(0)]); //only red on
    pwm.enable();

    // setup DIN for key leds - note only works if vhi is on
    // p.P0_15

    // setup accelerometer i2c
    // 6D_PWR: p.P0_08 -> output high to enable accelerometer
    // 6D_INT1: p.P0_11 -> interrupt input from accelerometer
    // 6D_i2C_SDA: p.P0_07
    // 6D_i2C_SCL: p.P0_27

    // setup GPIO to enable various keys
    let mut muxen01 = Output::new(p.P1_13, Level::High, OutputDrive::Standard);
    let mut muxen23 = Output::new(p.P1_14, Level::High, OutputDrive::Standard);
    let mut muxen45 = Output::new(p.P1_15, Level::High, OutputDrive::Standard);
    let mut muxens = [&mut muxen01, &mut muxen23, &mut muxen45];

    let mut mux_a = Output::new(p.P0_09, Level::Low, OutputDrive::Standard);
    let mut mux_b = Output::new(p.P0_10, Level::Low, OutputDrive::Standard);

    // setup ADC for key position reading
    let keyband0_channel_config = saadc::ChannelConfig::single_ended(p.P0_02.reborrow());
    let keyband1_channel_config = saadc::ChannelConfig::single_ended(p.P0_03.reborrow());
    let keyband2_channel_config = saadc::ChannelConfig::single_ended(p.P0_04.reborrow());
    let keyband3_channel_config = saadc::ChannelConfig::single_ended(p.P0_05.reborrow());
    let keyband4_channel_config = saadc::ChannelConfig::single_ended(p.P0_28.reborrow());
    let keyband5_channel_config = saadc::ChannelConfig::single_ended(p.P0_29.reborrow());
    let mut channel_configs = [
        keyband0_channel_config,
        keyband1_channel_config,
        keyband2_channel_config,
        keyband3_channel_config,
        keyband4_channel_config,
        keyband5_channel_config,
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
    pwm.set_all_duties([DutyCycle::normal(0), DutyCycle::normal(255), DutyCycle::normal(0), DutyCycle::normal(0)]); 
    

    loop { 
        let abpattern = [[Level::Low, Level::Low],
                                          [Level::High, Level::Low],
                                          [Level::Low, Level::High],
                                          [Level::High, Level::High]];

        let mut allbuff = [0i16; 6*4];
        
        let presample = Instant::now();

        for (index, ablevel) in abpattern.iter().enumerate() {
            for muxen in muxens.iter_mut() { muxen.set_high(); }
            mux_a.set_level(ablevel[0]);
            mux_b.set_level(ablevel[1]);
            for muxen in muxens.iter_mut() { muxen.set_low(); }

            let mut buf = [0i16; 6];
            adc.sample(&mut buf).await;

            allbuff[index*6..(index+1)*6].copy_from_slice(&buf);

        }

        let postsample = Instant::now();

        let midsample_micros = (presample.as_micros() + postsample.as_micros())/2;
        defmt::info!("reads from adc completed at {} sec", midsample_micros as f32 * 1e-6);

        for data in allbuff.iter() {
            let s = format!(7; "{},", data).expect("formatting failed");
            uart.write(s.as_bytes()).await.expect("uart couldn't write data");
        }
        uart.write(format!(23; "{}\r\n", midsample_micros).expect("ts formatting failed").as_bytes()).await.expect("uart couldn't write timestamp");
        
        hsv_on_board_leds(&mut pwm, ((postsample.as_millis() as f32)*(360./10000.)) % 360., 1.0, 1.0); //color cycle in 10 secs
    }
}