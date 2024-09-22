use tokio::{fs::File, io::AsyncReadExt};

use crate::registry::{MixerInput, MixerOutput, MixerOutputType, PipewirePorts};

pub async fn read_outputs_file(path: &str) -> Vec<MixerOutput> {
    match File::open(&path).await {
        Err(_why) => {
            vec![
                MixerOutput {
                    name: String::from("Main"),
                    pipewire_ports: PipewirePorts::None {},
                    id: 1,
                    output_type: MixerOutputType::Main,
                },
                MixerOutput {
                    name: String::from("Cue"),
                    pipewire_ports: PipewirePorts::None {},
                    id: 2,
                    output_type: MixerOutputType::Cue,
                },
                MixerOutput {
                    name: String::from("Main 2"),
                    pipewire_ports: PipewirePorts::None {},
                    id: 3,
                    output_type: MixerOutputType::Main,
                },
            ]
        }
        Ok(mut file) => {
            let mut raw_string = String::new();

            if let Err(why) = file.read_to_string(&mut raw_string).await {
                panic!("couldn't read {}", why);
            }

            serde_json::from_str(&raw_string).unwrap()
        }
    }
}

pub async fn read_inputs_file(path: &str) -> Vec<MixerInput> {
    match File::open(&path).await {
        Err(_why) => {
            vec![
                MixerInput {
                    name: String::from("DSMPL"),
                    pipewire_ports: crate::registry::PipewirePorts::None,
                    id: 1,
                    group_channel_strip_name: String::from("Drums"),
                },
                MixerInput {
                    name: String::from("DFire"),
                    pipewire_ports: crate::registry::PipewirePorts::None,
                    id: 2,
                    group_channel_strip_name: String::from("Drums"),
                },
                MixerInput {
                    name: String::from("DEuro"),
                    pipewire_ports: crate::registry::PipewirePorts::None,
                    id: 3,
                    group_channel_strip_name: String::from("Drums"),
                },
                MixerInput {
                    name: String::from("Prophet rev2"),
                    pipewire_ports: crate::registry::PipewirePorts::None,
                    id: 4,
                    group_channel_strip_name: String::from("Melody"),
                },
                MixerInput {
                    name: String::from("SE02"),
                    pipewire_ports: crate::registry::PipewirePorts::None,
                    id: 5,
                    group_channel_strip_name: String::from("Bass"),
                },
                MixerInput {
                    name: String::from("Torso S4"),
                    pipewire_ports: crate::registry::PipewirePorts::None,
                    id: 6,
                    group_channel_strip_name: String::from("Atmos"),
                },
                MixerInput {
                    name: String::from("opsix"),
                    pipewire_ports: crate::registry::PipewirePorts::None,
                    id: 7,
                    group_channel_strip_name: String::from("Drums"),
                },
                MixerInput {
                    name: String::from("System 1m"),
                    pipewire_ports: crate::registry::PipewirePorts::None,
                    id: 8,
                    group_channel_strip_name: String::from("Drums"),
                },
                MixerInput {
                    name: String::from("Cobalt 8m"),
                    pipewire_ports: crate::registry::PipewirePorts::None,
                    id: 9,
                    group_channel_strip_name: String::from("Drums"),
                },
            ]
        }
        Ok(mut file) => {
            let mut raw_string = String::new();

            if let Err(why) = file.read_to_string(&mut raw_string).await {
                panic!("couldn't read {}", why);
            }

            serde_json::from_str(&raw_string).unwrap()
        }
    }
}
