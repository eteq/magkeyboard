use crate::{KEYCHANGE_BUS_CAP, KEYCHANGE_BUS_SUBS};
use crate::hardware_consts::N_KEYS;

use embassy_nrf::gpio::Level;
use embassy_sync::pubsub::Publisher;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::lazy_lock::LazyLock;

use usbd_hid::descriptor::KeyboardUsage;

use heapless::index_map::FnvIndexMap;


#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Layer {
    Default
}
//const N_LAYERS: usize = mem::variant_count::<Layers>(); // not stabilized - https://github.com/rust-lang/rust/issues/73662
const N_LAYERS: usize = 1;


#[derive(Debug, Clone, Copy)]
pub struct KeySignal {
    pub toggle_on: bool,
    pub keynumber: u8,
}

#[derive(Debug)]
pub struct AnalogKey<M:RawMutex + 'static> 
{
    pub keynumber: u8,  // the "name" of the key - might not be sequential
    pub value: Option<f32>, // the most recent smoothed analog reading for this key
    pub filter_alpha: f32,
    pub max_value: Option<f32>,
    pub min_value: Option<f32>,
    switch_threshold: f32,
    pub switch_hysteresis_fraction: f32,
    pub high_is_on: bool,
    pub norm_valid_range: f32,
    pub toggle_publisher: Option<Publisher<'static, M, KeySignal, KEYCHANGE_BUS_CAP, KEYCHANGE_BUS_SUBS, N_KEYS>>,
} 

impl<M: RawMutex> AnalogKey<M> {
    pub fn new(keynumber: u8, toggle_publisher: Option<Publisher<'static, M, KeySignal, KEYCHANGE_BUS_CAP, KEYCHANGE_BUS_SUBS, N_KEYS>>) -> Self {
        AnalogKey {
            keynumber: keynumber,
            value: None,
            filter_alpha: 0.05,
            max_value: None,
            min_value: None,
            switch_threshold: 0.5,
            switch_hysteresis_fraction: 0.1,
            high_is_on: false,
            norm_valid_range: 100.,
            toggle_publisher: toggle_publisher,
        }
    }
    pub fn update_value_adc(&mut self, new_adc_value: i16) {
        let oldon = self.is_on();

        if let Some(oldval) = self.value {
            let newval = (1. - self.filter_alpha) * oldval + self.filter_alpha * (new_adc_value as f32);

            if let Some(maxval) = self.max_value {
                if newval > maxval {
                    self.max_value = Some(newval);
                }
            } else {
                self.max_value = Some(newval);
            }
            if let Some(minval) = self.min_value {
                if newval < minval {
                    self.min_value = Some(newval);
                }
            } else {
                self.min_value = Some(newval);
            }

            self.value = Some(newval);

            if oldval < self.switch_threshold && newval >= self.switch_threshold {
                // crossed threshold upwards
                self.switch_threshold = 0.5 - self.switch_hysteresis_fraction;
            } else if oldval >= self.switch_threshold && newval < self.switch_threshold {
                // crossed threshold downwards
                self.switch_threshold = 0.5 + self.switch_hysteresis_fraction;
            }
        } else {
            self.value = Some(new_adc_value as f32);
        }

        let newon = self.is_on();
        match (oldon, newon) {
            (Some(old), Some(new)) if old != new => {
                self.toggled(new);
            }
            _ => {}
        }
    }

    fn toggled(&self, to_on: bool) {
        match &self.toggle_publisher {
            Some(publisher) => {
                let signal = KeySignal {
                    toggle_on: to_on,
                    keynumber: self.keynumber,
                };
                // we use try_publish to avoid blocking here, since that could cause missed ADC readings
                match publisher.try_publish(signal) {
                    Ok(()) => {}
                    Err(_sig) => {
                        // right now this probably means the USB isn't on.  Should check for that and not worry if it's disconnected
                        defmt::warn!("Failed to publish key toggle signal for key {}", self.keynumber);
                    }
                }
            }
            None => {}
        }
    }

    pub fn normalized_value(&self) -> Option<f32> {
        if self.value.is_none() {
            return None;
        }

        match (self.min_value, self.max_value) {
            (Some(minval), Some(maxval)) if maxval > minval => {
                if (maxval - minval) < self.norm_valid_range { return None; }
                return Some((self.value.unwrap() - minval) / (maxval - minval));
            }
            _ => { return None; }
        }
    }

    pub fn is_on(&self) -> Option<bool> {
        let normval = self.normalized_value()?;
        let high = normval >= self.switch_threshold;

        if self.high_is_on {
            Some(high)
        } else {
            Some(!high)
        }
    }
}

impl<M: RawMutex> Default for AnalogKey<M> {
    fn default() -> Self { AnalogKey::<M>::new(0, None) }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MuxSpec {
    // the Mux configuration for a specific key
    pub a: Level,
    pub b: Level,
}

impl MuxSpec {
    pub fn iterator() -> core::slice::Iter<'static, MuxSpec> {
        static ALLSPECS: [MuxSpec; 4] = [MuxSpec { a: Level::Low, b: Level::Low }, 
                                         MuxSpec { a: Level::High, b: Level::Low }, 
                                         MuxSpec { a: Level::Low, b: Level::High }, 
                                         MuxSpec { a: Level::High, b: Level::High }];
        ALLSPECS.iter()
    }
    pub fn index(&self) -> u8 {
        match (self.a, self.b) {
            (Level::Low, Level::Low) => 0,
            (Level::High, Level::Low) => 1,
            (Level::Low, Level::High) => 2,
            (Level::High, Level::High) => 3,
        }
    }
}

impl Default for MuxSpec {
    fn default() -> Self { MuxSpec { a: Level::Low, b: Level::Low } }
}


const N_KEYMAP: usize = N_KEYS * N_LAYERS;
const N_KEYMAP_POWEROF2: usize = N_KEYMAP.next_power_of_two();
pub static KEYMAP: LazyLock<FnvIndexMap<(u8, Layer), KeyboardUsage, N_KEYMAP_POWEROF2>> = LazyLock::new(|| {
    let mut m = FnvIndexMap::new();
    m.insert((00, Layer::Default), KeyboardUsage::KeyboardQq).expect("no space for key!");
    m.insert((01, Layer::Default), KeyboardUsage::KeyboardWw).expect("no space for key!");
    m.insert((02, Layer::Default), KeyboardUsage::KeyboardEe).expect("no space for key!");
    m.insert((03, Layer::Default), KeyboardUsage::KeyboardRr).expect("no space for key!");
    m.insert((10, Layer::Default), KeyboardUsage::KeyboardTt).expect("no space for key!");
    m.insert((11, Layer::Default), KeyboardUsage::KeypadTab).expect("no space for key!");
    m.insert((12, Layer::Default), KeyboardUsage::KeyboardAa).expect("no space for key!");
    m.insert((13, Layer::Default), KeyboardUsage::KeyboardSs).expect("no space for key!");
    m.insert((20, Layer::Default), KeyboardUsage::KeyboardDd).expect("no space for key!");
    m.insert((21, Layer::Default), KeyboardUsage::KeyboardFf).expect("no space for key!");
    m.insert((22, Layer::Default), KeyboardUsage::KeyboardGg).expect("no space for key!");
    m.insert((23, Layer::Default), KeyboardUsage::KeypadLeftShift).expect("no space for key!");
    m.insert((30, Layer::Default), KeyboardUsage::KeyboardZz).expect("no space for key!");
    m.insert((31, Layer::Default), KeyboardUsage::KeyboardXx).expect("no space for key!");
    m.insert((32, Layer::Default), KeyboardUsage::KeyboardCc).expect("no space for key!");
    m.insert((33, Layer::Default), KeyboardUsage::KeyboardVv).expect("no space for key!");
    m.insert((40, Layer::Default), KeyboardUsage::KeyboardBb).expect("no space for key!");
    m.insert((41, Layer::Default), KeyboardUsage::KeyboardLeftControl).expect("no space for key!");
    m.insert((42, Layer::Default), KeyboardUsage::KeyboardLeftAlt).expect("no space for key!");
    //m.insert((43, Layer::Default), KeyboardUsage::Keyboard1<FIX>).expect("no space for key!");
    //m.insert((50, Layer::Default), KeyboardUsage::Keyboard2<FIX>).expect("no space for key!");
    m.insert((51, Layer::Default), KeyboardUsage::KeyboardSpacebar).expect("no space for key!");
    //m.insert((52, Layer::Default), KeyboardUsage::Keyboard3<FIX>).expect("no space for key!");

    m
});