#[macro_use]
extern crate vst;
use std::sync::Arc;
use vst::prelude::*;

const SAMPLE_RATE : f64 = 44100.0;
const MAX_DELAY_TIME : f64 = 10.0;
const DELAY_LINE_LEN : usize = MAX_DELAY_TIME as usize * SAMPLE_RATE as usize;


struct doubleDelay {
    // Store a handle to the plugin's parameter object.
    params: Arc<DelayEffectParameters>,
    delay_line: [f32;DELAY_LINE_LEN],
    delay_index: usize,
}


/// The plugin's parameter object contains the values of parameters that can be
/// adjusted from the host.  If we were creating an effect that didn't allow the
/// user to modify it at runtime or have any controls, we could omit this part.
///
/// The parameters object is shared between the processing and GUI threads.
/// For this reason, all mutable state in the object has to be represented
/// through thread-safe interior mutability. The easiest way to achieve this
/// is to store the parameters in atomic containers.
struct DelayEffectParameters {
    // The plugin's state consists of a single parameter: amplitude.
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
            delay_time: AtomicFloat::new(1.0),
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
        // Note that controls will always return a value from 0 - 1.
        // Setting a default to 0.5 means it's halfway up.
        doubleDelay {
            params: Arc::new(DelayEffectParameters::default()),
            delay_line: [0.0;DELAY_LINE_LEN],
            delay_index: 0,
        }
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Delay in Rust".to_string(),
            vendor: "Sam Segal".to_string(),
            unique_id: 243723072,
            version: 1,
            inputs: 2,
            outputs: 2,
            // This `parameters` bit is important; without it, none of our
            // parameters will be shown!
            parameters: 5,
            category: Category::Effect,
            ..Default::default()
        }
    }

    // Here is where the bulk of our audio processing code goes.
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // Read the amplitude from the parameter object
        let amplitude = self.params.amplitude.get();
        let drymix = self.params.delay_dry.get();
        let wetmix = self.params.delay_wet.get();
        let feedback = self.params.delay_feedback.get();
        let samples_len = self.params.delay_time.get() * SAMPLE_RATE as f32;
        let  mut delay_buffer = self.delay_line ;
        let mut delay_index = self.delay_index ;
        // First, we destructure our audio buffer into an arbitrary number of
        // input and output buffers.  Usually, we'll be dealing with stereo (2 of each)
        // but that might change.
        for (input_buffer, output_buffer) in buffer.zip() {
            // Next, we'll loop through each individual sample so we can apply the amplitude
            // value to it.
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
            let delayed = delay_buffer[delay_index];
            *output_sample = ((*input_sample * drymix) + (delayed * wetmix)) * amplitude;
            

            delay_buffer[delay_index] = feedback + (delayed * *input_sample);
            delay_index += 1 ;
            if delay_index >= samples_len as usize {
                delay_index = 0
            }

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
            _ => 0.0,
        }
    }

    // the `set_parameter` function sets the value of a parameter.
    fn set_parameter(&self, index: i32, val: f32) {
        #[allow(clippy::single_match)]
        match index {
            0 => self.amplitude.set(val),
            1 => self.delay_feedback.set(val),
            2 => self.delay_time.set(val),
            _ => (),
        }
    }

    // This is what will display underneath our control.  We can
    // format it into a string that makes the most since.
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.2}", (self.amplitude.get() - 0.5) * 2f32),
            1 => format!("{:.2}", (self.delay_feedback.get()) * 2f32),
            2 => format!("{:.2}", (self.delay_time.get()) * 2f32), 
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