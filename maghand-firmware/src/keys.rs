use embassy_nrf::gpio::Level;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AnalogKey {
    pub keynumber: u8,  // the "name" of the key - might not be sequential
    pub value: Option<f32>, // the most recent smoothed analog reading for this key
    pub filter_alpha: f32,
    pub max_value: Option<f32>,
    pub min_value: Option<f32>,
    switch_threshold: f32,
    pub switch_hysteresis_fraction: f32,
    pub high_is_on: bool,
    pub norm_valid_range: f32,

} 

impl AnalogKey {
    pub fn new(keynumber: u8) -> Self {
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
        }
    }
    pub fn update_value_adc(&mut self, new_adc_value: i16) {
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

impl Default for AnalogKey {
    fn default() -> Self { AnalogKey::new(0) }
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