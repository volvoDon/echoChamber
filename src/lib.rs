#[macro_use]
extern crate vst;
use std::sync::Arc;
use vst::prelude::*;

struct doubleDelay {
    // Store a handle to the plugin's parameter object.
    params: Arc<DelayEffectParameters>,
    delay_line: [f32;441010],
    delay_index: usize,
}
// fn to access and maniputlate the delay_lines 
impl doubleDelay {
    
    fn set_index (&mut self, sample_len: f32) {
        if self.delay_index >= sample_len.round() as usize {self.delay_index = 0}
        else {self.delay_index += 1}
        
    }
    fn get_delay (&self) -> f32 {
        self.delay_line[self.delay_index]
    }
    fn set_delay (&mut self,new: f32) {
       self.delay_line[self.delay_index] = new 
    }
}

struct DelayEffectParameters {
    amplitude: AtomicFloat,
    delay_time: AtomicFloat,
    delay_wet: AtomicFloat,
    delay_dry: AtomicFloat,
    delay_feedback: AtomicFloat,
}

impl Default for DelayEffectParameters {
    fn default() -> DelayEffectParameters {
        DelayEffectParameters {
            amplitude: AtomicFloat::new(0.5),
            delay_time: AtomicFloat::new(0.5),
            delay_wet: AtomicFloat::new(0.5),
            delay_dry: AtomicFloat::new(0.5),
            delay_feedback: AtomicFloat::new(0.3),
        }
    }
}

// All plugins using `vst` also need to implement the `Plugin` trait.  Here, we
// define functions that give necessary info to our host.
impl Plugin for doubleDelay {
    fn new(_host: HostCallback) -> Self {
        // controls will always return a value from 0 - 1.
        // Setting a default to 0.5 means it's halfway
        doubleDelay {
            params: Arc::new(DelayEffectParameters::default()),
            delay_line: [0.0;441010],
            delay_index: 0,
        }
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Delay in Rust".to_string(),
            vendor: "Sam Segal".to_string(),
            unique_id: 11458734,
            version: 1,
            inputs: 2,
            outputs: 2,
            parameters: 3,
            category: Category::Effect,
            ..Default::default()
        }
    }

    //  audio processing code
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let amp = self.params.amplitude.get();
        let drymix = self.params.delay_dry.get();
        let wetmix = self.params.delay_wet.get();
        let feedback = self.params.delay_feedback.get();
        let sample_len = self.params.delay_time.get() * 44100.0 ;
        
        for (input_buffer, output_buffer) in buffer.zip() {
            // Next, we'll loop through each individual sample so we can apply the amplitude
            // value to it.
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
            *output_sample = ((*input_sample * drymix) + (wetmix * self.get_delay()))*amp;
            let new = (*input_sample + self.get_delay()) * feedback;
            self.set_delay(new);
            self.set_index(sample_len);

            }
        }
    }

    // Return the parameter object. This method can be omitted if the
    // plugin has no parameters.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl PluginParameters for DelayEffectParameters {
    // the `get_parameter` function reads the value of a parameter.
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.amplitude.get(),
            1 => self.delay_feedback.get(),
            2 => self.delay_time.get(),
            _ => 0.0,
        }
    }

    // the `set_parameter` function sets the value of a parameter.
    fn set_parameter(&self, index: i32, val: f32) {
        #[allow(clippy::single_match)]
        match index {
            0 => self.amplitude.set(val),
            1 => self.delay_feedback.set(val),
            2 => self.delay_time.set(val * 4.0),
            _ => (),
        }
    }

    // This is what will display underneath our control.  We can
    // format it into a string that makes the most since.
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.2}", (self.amplitude.get() - 0.5) * 2f32),
            1 => format!("{:.2}", (self.delay_feedback.get())),
            2 => format!("{:.2}", (self.delay_time.get())*4f32), 
            _ => "".to_string(),
        }
    }

    // This shows the control's name.
    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Amplitude",
            1 => "feedback",
            2 => "Time",
            _ => "",
        }
        .to_string()
    }
}

// This part is important!  Without it, our plugin won't work.
plugin_main!(doubleDelay);