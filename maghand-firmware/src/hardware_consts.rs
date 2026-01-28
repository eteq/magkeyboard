use embassy_time::Duration;

// note the order here should match the order of the leds
pub const KEY_NAMES: [u8; 23] = [00, 01, 02, 03, 
                             10, 11, 12, 13,
                             20, 21, 22, 23,
                             30, 31, 32, 33,
                             40, 41, 42, 43,
                             50, 51, 52];//, 53];
pub const N_KEYS: usize = KEY_NAMES.len();

pub const LED_POWERUP_TIME: Duration = Duration::from_millis(1); // this is just a guess - implicitly it's everything connected to vhi
pub const IMU_POWERUP_TIME: Duration = Duration::from_millis(35); // lsm6ds3tr datasheet
pub const MUX_SETTLE_TIME: Option<Duration> = Some(Duration::from_millis(1)); // lsm6ds3tr datasheet