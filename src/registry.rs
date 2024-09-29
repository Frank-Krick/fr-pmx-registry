use serde::{Deserialize, Serialize};

use crate::pmx::{
    channel_strip::{PmxChannelStrip, PmxChannelStripType},
    looper::PmxLooper,
    output_stage::PmxOutputStage,
    plugin::PmxPlugin,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MixerInput {
    pub name: String,
    pub pipewire_ports: PipewirePorts,
    pub id: u32,
    pub group_channel_strip_name: String,
}

impl MixerInput {
    pub fn new(
        name: &str,
        pipewire_ports: PipewirePorts,
        id: u32,
        group_channel_strip_name: &str,
    ) -> Self {
        MixerInput {
            id,
            name: String::from(name),
            pipewire_ports,
            group_channel_strip_name: String::from(group_channel_strip_name),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MixerOutputType {
    Cue,
    Main,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MixerOutput {
    pub name: String,
    pub pipewire_ports: PipewirePorts,
    pub id: u32,
    pub output_type: MixerOutputType,
}

impl MixerOutput {
    pub fn new(
        name: &str,
        pipewire_ports: PipewirePorts,
        id: u32,
        output_type: MixerOutputType,
    ) -> Self {
        MixerOutput {
            id,
            name: String::from(name),
            pipewire_ports,
            output_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipewirePorts {
    None,
    Mono(String),
    Stereo(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginType {
    Lv2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: u32,
    pub mod_host_id: u32,
    pub name: String,
    pub plugin_uri: String,
    pub plugin_type: PluginType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelStripType {
    Basic {
        saturator_plugin_id: u32,
        compressor_plugin_id: u32,
        equalizer_plugin_id: u32,
        gain_plugin_id: u32,
    },
    CrossFaded {
        cross_fader_plugin_id: u32,
        saturator_plugin_id: u32,
        compressor_plugin_id: u32,
        equalizer_plugin_id: u32,
        gain_plugin_id: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStrip {
    pub id: u32,
    pub name: String,
    pub channel_strip_type: ChannelStripType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputStage {
    pub id: u32,
    pub name: String,
    pub left_channel_strip_id: u32,
    pub right_channel_strip_id: u32,
    pub cross_fader_plugin_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Looper {
    pub id: u32,
    pub name: String,
    pub loop_number: u32,
}

#[derive(Debug)]
pub struct Registry {
    inputs: Vec<MixerInput>,
    outputs: Vec<MixerOutput>,
    inputs_sender: tokio::sync::mpsc::UnboundedSender<Vec<MixerInput>>,
    outputs_sender: tokio::sync::mpsc::UnboundedSender<Vec<MixerOutput>>,
    plugins: Vec<Plugin>,
    channel_strips: Vec<ChannelStrip>,
    loopers: Vec<Looper>,
    output_stages: Vec<OutputStage>,
}

impl Registry {
    pub fn new(
        inputs: Vec<MixerInput>,
        outputs: Vec<MixerOutput>,
        inputs_sender: tokio::sync::mpsc::UnboundedSender<Vec<MixerInput>>,
        outputs_sender: tokio::sync::mpsc::UnboundedSender<Vec<MixerOutput>>,
    ) -> Self {
        Registry {
            inputs,
            outputs,
            inputs_sender,
            outputs_sender,
            plugins: Vec::new(),
            channel_strips: Vec::new(),
            loopers: Vec::new(),
            output_stages: Vec::new(),
        }
    }

    pub fn register_output_stage(&mut self, output_stage: PmxOutputStage) {
        self.output_stages.push(OutputStage {
            id: output_stage.id,
            name: output_stage.name,
            left_channel_strip_id: output_stage.left_channel_strip_id,
            right_channel_strip_id: output_stage.right_channel_strip_id,
            cross_fader_plugin_id: output_stage.cross_fader_plugin_id,
        });
    }

    pub fn get_all_output_stages(&self) -> &[OutputStage] {
        &self.output_stages
    }

    pub fn register_looper(&mut self, looper: PmxLooper) {
        self.loopers.push(Looper {
            id: looper.id,
            name: looper.name,
            loop_number: looper.loop_number,
        });
    }

    pub fn get_all_loopers(&self) -> &Vec<Looper> {
        &self.loopers
    }

    pub fn get_looper_by_id(&self, id: u32) -> Option<&Looper> {
        self.loopers.iter().find(|c| c.id == id)
    }

    pub fn register_channel_strip(&mut self, channel_strip: PmxChannelStrip) {
        self.channel_strips.push(ChannelStrip {
            id: channel_strip.id,
            name: channel_strip.name.clone(),
            channel_strip_type: match channel_strip.channel_strip_type() {
                PmxChannelStripType::Basic => ChannelStripType::Basic {
                    saturator_plugin_id: channel_strip.saturator_plugin_id,
                    compressor_plugin_id: channel_strip.compressor_plugin_id,
                    equalizer_plugin_id: channel_strip.equalizer_plugin_id,
                    gain_plugin_id: channel_strip.gain_plugin_id,
                },
                PmxChannelStripType::CrossFaded => ChannelStripType::CrossFaded {
                    cross_fader_plugin_id: channel_strip.cross_fader_plugin_id.unwrap(),
                    saturator_plugin_id: channel_strip.saturator_plugin_id,
                    compressor_plugin_id: channel_strip.compressor_plugin_id,
                    equalizer_plugin_id: channel_strip.equalizer_plugin_id,
                    gain_plugin_id: channel_strip.gain_plugin_id,
                },
            },
        });
    }

    pub fn get_channel_strip_by_id(&self, id: u32) -> Option<&ChannelStrip> {
        self.channel_strips.iter().find(|c| c.id == id)
    }

    pub fn register_plugin(&mut self, plugin: PmxPlugin) {
        self.plugins.push(Plugin {
            id: plugin.id,
            mod_host_id: plugin.mod_host_id,
            name: plugin.name,
            plugin_uri: plugin.plugin_uri,
            plugin_type: PluginType::Lv2,
        });
    }

    pub fn get_plugin_by_id(&self, id: u32) -> Option<&Plugin> {
        self.plugins.iter().find(|p| p.id == id)
    }

    pub fn get_all_plugins(&self) -> &Vec<Plugin> {
        &self.plugins
    }

    pub fn get_all_outputs(&self) -> &[MixerOutput] {
        &self.outputs
    }

    pub fn output_by_id(&self, id: u32) -> Option<&MixerOutput> {
        self.outputs.iter().find(|o| o.id == id)
    }

    pub fn update_output_ports(
        &mut self,
        id: u32,
        ports: PipewirePorts,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(output) = self
            .outputs
            .clone()
            .iter()
            .enumerate()
            .find(|(_index, output)| output.id == id)
        {
            self.outputs[output.0].pipewire_ports = ports;
            self.outputs_sender.send(self.outputs.clone()).unwrap();
            Ok(())
        } else {
            Err(std::boxed::Box::new(NotFoundError {}))
        }
    }

    pub fn get_all_inputs(&self) -> &[MixerInput] {
        &self.inputs
    }

    pub fn input_by_id(&self, id: u32) -> Option<&MixerInput> {
        self.inputs.iter().find(|i| i.id == id)
    }

    pub fn update_input_name(
        &mut self,
        id: u32,
        name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(input) = self
            .inputs
            .clone()
            .iter()
            .enumerate()
            .find(|(_index, input)| input.id == id)
        {
            self.inputs[input.0].name = String::from(name);
            self.inputs_sender.send(self.inputs.clone()).unwrap();
            Ok(())
        } else {
            Err(std::boxed::Box::new(NotFoundError {}))
        }
    }

    pub fn update_input_ports(
        &mut self,
        id: u32,
        ports: PipewirePorts,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(input) = self
            .inputs
            .clone()
            .iter()
            .enumerate()
            .find(|(_index, input)| input.id == id)
        {
            self.inputs[input.0].pipewire_ports = ports;
            self.inputs_sender.send(self.inputs.clone()).unwrap();
            Ok(())
        } else {
            Err(std::boxed::Box::new(NotFoundError {}))
        }
    }

    pub fn get_all_channel_strips(&self) -> &Vec<ChannelStrip> {
        &self.channel_strips
    }
}

#[derive(Debug)]
struct NotFoundError {}

impl std::fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("couldn't find input")?;
        Ok(())
    }
}

impl std::error::Error for NotFoundError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}
