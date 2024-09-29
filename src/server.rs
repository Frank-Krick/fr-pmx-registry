use pmx::output::{PmxOutput, PmxOutputType};
use registry::{MixerInput, MixerOutput};
use std::result::Result;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};

use pmx::channel_strip::{PmxChannelStrip, PmxChannelStripType};
use pmx::input::{PmxInput, PmxInputType};
use pmx::looper::PmxLooper;
use pmx::output_stage::PmxOutputStage;
use pmx::plugin::{PmxPlugin, PmxPluginType};
use pmx::pmx_registry_server::{PmxRegistry, PmxRegistryServer};
use pmx::{
    ByIdRequest, EmptyRequest, ListChannelStripsReply, ListInputsReply, ListLoopersReply,
    ListOutputStagesReply, ListOutputsReply, ListPluginsReply, RegisterChannelStripRequest,
    RegisterLooperRequest, RegisterOutputStageRequest, RegisterPluginRequest,
    UpdateInputNameRequest, UpdateInputPortAssignmentsRequest, UpdateOutputPortAssignmentsRequest,
};

use crate::registry::{PipewirePorts, Registry};

pub mod pmx {
    tonic::include_proto!("pmx");

    pub mod input {
        tonic::include_proto!("pmx.input");
    }

    pub mod output {
        tonic::include_proto!("pmx.output");
    }

    pub mod plugin {
        tonic::include_proto!("pmx.plugin");
    }

    pub mod channel_strip {
        tonic::include_proto!("pmx.channel_strip");
    }

    pub mod looper {
        tonic::include_proto!("pmx.looper");
    }

    pub mod output_stage {
        tonic::include_proto!("pmx.output_stage");
    }
}

mod file_reader;
mod file_writer;
mod registry;

#[derive(Debug)]
pub struct PmxRegistryService {
    registry: RwLock<Registry>,
}

impl PmxRegistryService {
    fn new(
        inputs: Vec<MixerInput>,
        outputs: Vec<MixerOutput>,
        inputs_sender: tokio::sync::mpsc::UnboundedSender<Vec<MixerInput>>,
        outputs_sender: tokio::sync::mpsc::UnboundedSender<Vec<MixerOutput>>,
    ) -> Self {
        PmxRegistryService {
            registry: RwLock::new(Registry::new(
                inputs,
                outputs,
                inputs_sender,
                outputs_sender,
            )),
        }
    }
}

impl PmxInput {
    fn from(input: &MixerInput) -> Self {
        PmxInput {
            id: input.id,
            name: input.name.clone(),
            input_type: match input.pipewire_ports {
                PipewirePorts::None => PmxInputType::None as i32,
                PipewirePorts::Mono(_) => PmxInputType::MonoInput as i32,
                PipewirePorts::Stereo(_, _) => PmxInputType::StereoInput as i32,
            },
            left_port_path: match input.pipewire_ports.clone() {
                PipewirePorts::Mono(path) => Some(path),
                PipewirePorts::Stereo(path, _) => Some(path),
                _ => None,
            },
            right_port_path: match input.pipewire_ports.clone() {
                PipewirePorts::Stereo(_, path) => Some(path),
                _ => None,
            },
            group_channel_strip_name: input.group_channel_strip_name.clone(),
        }
    }
}

#[tonic::async_trait]
impl PmxRegistry for PmxRegistryService {
    async fn list_channel_strips(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<ListChannelStripsReply>, Status> {
        let registry = self.registry.read().await;
        let channel_strips = registry.get_all_channel_strips();

        Ok(Response::new(ListChannelStripsReply {
            channel_strips: channel_strips
                .iter()
                .map(|c| match c.channel_strip_type {
                    registry::ChannelStripType::Basic {
                        saturator_plugin_id,
                        compressor_plugin_id,
                        equalizer_plugin_id,
                        gain_plugin_id,
                    } => PmxChannelStrip {
                        id: c.id,
                        name: c.name.clone(),
                        channel_strip_type: PmxChannelStripType::Basic as i32,
                        cross_fader_plugin_id: None,
                        saturator_plugin_id,
                        compressor_plugin_id,
                        equalizer_plugin_id,
                        gain_plugin_id,
                    },
                    registry::ChannelStripType::CrossFaded {
                        cross_fader_plugin_id,
                        saturator_plugin_id,
                        compressor_plugin_id,
                        equalizer_plugin_id,
                        gain_plugin_id,
                    } => PmxChannelStrip {
                        id: c.id,
                        name: c.name.clone(),
                        channel_strip_type: PmxChannelStripType::CrossFaded as i32,
                        cross_fader_plugin_id: Some(cross_fader_plugin_id),
                        saturator_plugin_id,
                        compressor_plugin_id,
                        equalizer_plugin_id,
                        gain_plugin_id,
                    },
                })
                .collect(),
        }))
    }

    async fn list_inputs(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<ListInputsReply>, Status> {
        let registry = self.registry.read().await;
        let inputs = registry.get_all_inputs().iter().map(PmxInput::from);

        Ok(Response::new(ListInputsReply {
            inputs: inputs.collect(),
        }))
    }

    async fn get_input(&self, request: Request<ByIdRequest>) -> Result<Response<PmxInput>, Status> {
        let id = request.into_inner().id;
        let registry = self.registry.read().await;
        match registry.input_by_id(id) {
            Some(input) => Ok(Response::new(PmxInput::from(input))),
            None => Err(Status::not_found(format!(
                "Couldn't find input with id: {id}"
            ))),
        }
    }

    async fn update_input_name(
        &self,
        request: Request<UpdateInputNameRequest>,
    ) -> Result<Response<PmxInput>, Status> {
        let inner = request.into_inner();
        let id = inner.id;
        let name = inner.name;
        let mut registry = self.registry.write().await;
        match registry.update_input_name(id, name.as_str()) {
            Ok(_) => {
                let input = registry.input_by_id(id).unwrap();
                Ok(Response::new(PmxInput::from(input)))
            }
            Err(_) => Err(Status::not_found(format!(
                "Couldn't find input with id: {id}"
            ))),
        }
    }

    async fn update_input_port_assignments(
        &self,
        request: Request<UpdateInputPortAssignmentsRequest>,
    ) -> Result<Response<PmxInput>, Status> {
        let inner = request.into_inner();
        let id = inner.id;
        let left_path = inner.left_port_path;
        let right_path = inner.right_port_path;
        match inner.input_type {
            x if x == PmxInputType::MonoInput as i32 => {
                let mut registry = self.registry.write().await;
                match registry.update_input_ports(id, PipewirePorts::Mono(left_path.unwrap())) {
                    Ok(_) => {
                        let input = registry.input_by_id(id).unwrap();
                        Ok(Response::new(PmxInput::from(input)))
                    }
                    Err(_) => Err(Status::not_found(format!(
                        "Couldn't find input with id: {id}"
                    ))),
                }
            }
            x if x == PmxInputType::StereoInput as i32 => {
                let mut registry = self.registry.write().await;
                match registry.update_input_ports(
                    id,
                    PipewirePorts::Stereo(left_path.unwrap(), right_path.unwrap()),
                ) {
                    Ok(_) => {
                        let input = registry.input_by_id(id).unwrap();
                        Ok(Response::new(PmxInput::from(input)))
                    }
                    Err(_) => Err(Status::not_found(format!(
                        "Couldn't find input with id: {id}"
                    ))),
                }
            }
            x if x == PmxInputType::None as i32 => {
                let mut registry = self.registry.write().await;
                match registry.update_input_ports(id, PipewirePorts::None) {
                    Ok(_) => {
                        let input = registry.input_by_id(id).unwrap();
                        Ok(Response::new(PmxInput::from(input)))
                    }
                    Err(_) => Err(Status::not_found(format!(
                        "Couldn't find input with id: {id}"
                    ))),
                }
            }
            _ => Err(Status::invalid_argument("invalid input type code")),
        }
    }

    async fn register_channel_strip(
        &self,
        request: Request<RegisterChannelStripRequest>,
    ) -> Result<Response<PmxChannelStrip>, Status> {
        let mut registry = self.registry.write().await;
        let channel_strip_to_register = request.into_inner().channel_strip.unwrap();
        let id = channel_strip_to_register.id;
        registry.register_channel_strip(channel_strip_to_register);
        if let Some(channel_strip) = registry.get_channel_strip_by_id(id) {
            Ok(Response::new(match channel_strip.channel_strip_type {
                registry::ChannelStripType::Basic {
                    saturator_plugin_id,
                    compressor_plugin_id,
                    equalizer_plugin_id,
                    gain_plugin_id,
                } => PmxChannelStrip {
                    id: channel_strip.id,
                    name: channel_strip.name.clone(),
                    channel_strip_type: PmxChannelStripType::Basic as i32,
                    cross_fader_plugin_id: None,
                    saturator_plugin_id,
                    compressor_plugin_id,
                    equalizer_plugin_id,
                    gain_plugin_id,
                },
                registry::ChannelStripType::CrossFaded {
                    cross_fader_plugin_id,
                    saturator_plugin_id,
                    compressor_plugin_id,
                    equalizer_plugin_id,
                    gain_plugin_id,
                } => PmxChannelStrip {
                    id: channel_strip.id,
                    name: channel_strip.name.clone(),
                    channel_strip_type: PmxChannelStripType::CrossFaded as i32,
                    cross_fader_plugin_id: Some(cross_fader_plugin_id),
                    saturator_plugin_id,
                    compressor_plugin_id,
                    equalizer_plugin_id,
                    gain_plugin_id,
                },
            }))
        } else {
            Err(Status::not_found("Channel Strip wasn't registered"))
        }
    }

    async fn register_plugin(
        &self,
        request: Request<RegisterPluginRequest>,
    ) -> Result<Response<PmxPlugin>, Status> {
        let mut registry = self.registry.write().await;
        let plugin_to_register = request.into_inner().plugin.unwrap();
        let id = plugin_to_register.id;
        registry.register_plugin(plugin_to_register);
        if let Some(plugin) = registry.get_plugin_by_id(id) {
            Ok(Response::new(PmxPlugin {
                id: plugin.id,
                mod_host_id: plugin.mod_host_id,
                name: plugin.name.clone(),
                plugin_uri: plugin.plugin_uri.clone(),
                plugin_type: match plugin.plugin_type {
                    registry::PluginType::Lv2 => PmxPluginType::Lv2 as i32,
                },
            }))
        } else {
            Err(Status::not_found(String::from("Plugin was not created")))
        }
    }

    async fn list_plugins(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<ListPluginsReply>, Status> {
        let registry = self.registry.read().await;
        let plugins = registry.get_all_plugins();
        Ok(Response::new(ListPluginsReply {
            plugins: plugins
                .iter()
                .map(|p| PmxPlugin {
                    id: p.id,
                    mod_host_id: p.mod_host_id,
                    name: p.name.clone(),
                    plugin_uri: p.plugin_uri.clone(),
                    plugin_type: match p.plugin_type {
                        registry::PluginType::Lv2 => PmxPluginType::Lv2 as i32,
                    },
                })
                .collect(),
        }))
    }

    async fn register_looper(
        &self,
        request: Request<RegisterLooperRequest>,
    ) -> Result<Response<PmxLooper>, Status> {
        let inner = request.into_inner();
        let mut registry = self.registry.write().await;
        registry.register_looper(PmxLooper {
            id: inner.loop_number,
            name: format!("loop_{}", inner.loop_number),
            loop_number: inner.loop_number,
        });
        let looper = registry.get_looper_by_id(inner.loop_number).unwrap();
        Ok(Response::new(PmxLooper {
            id: looper.id,
            name: looper.name.clone(),
            loop_number: looper.loop_number,
        }))
    }

    async fn list_loopers(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<ListLoopersReply>, Status> {
        let registry = self.registry.read().await;
        let loopers = registry.get_all_loopers();
        Ok(Response::new(ListLoopersReply {
            loopers: loopers
                .iter()
                .map(|l| PmxLooper {
                    id: l.id,
                    name: l.name.clone(),
                    loop_number: l.loop_number,
                })
                .collect(),
        }))
    }

    async fn list_outputs(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<ListOutputsReply>, Status> {
        let registry = self.registry.read().await;
        let outputs = registry.get_all_outputs();
        Ok(Response::new(ListOutputsReply {
            outputs: outputs
                .iter()
                .map(|o| PmxOutput {
                    id: o.id,
                    name: o.name.clone(),
                    output_type: match o.output_type {
                        registry::MixerOutputType::Cue => PmxOutputType::Cue as i32,
                        registry::MixerOutputType::Main => PmxOutputType::Main as i32,
                    },
                    left_port_path: match o.pipewire_ports.clone() {
                        PipewirePorts::None => None,
                        PipewirePorts::Mono(left) => Some(left),
                        PipewirePorts::Stereo(left, _) => Some(left),
                    },
                    right_port_path: match o.pipewire_ports.clone() {
                        PipewirePorts::None => None,
                        PipewirePorts::Mono(left) => Some(left),
                        PipewirePorts::Stereo(_, right) => Some(right),
                    },
                })
                .collect(),
        }))
    }

    async fn update_output_port_assignments(
        &self,
        request: Request<UpdateOutputPortAssignmentsRequest>,
    ) -> Result<Response<PmxOutput>, Status> {
        let inner = request.into_inner();
        let pipewire_ports = match (inner.left_port_path, inner.right_port_path) {
            (None, None) => PipewirePorts::None,
            (None, Some(right)) => PipewirePorts::Mono(right),
            (Some(left), None) => PipewirePorts::Mono(left),
            (Some(left), Some(right)) => PipewirePorts::Stereo(left, right),
        };
        let mut registry = self.registry.write().await;
        registry
            .update_output_ports(inner.id, pipewire_ports)
            .unwrap();
        let output = registry.output_by_id(inner.id).unwrap();
        Ok(Response::new(PmxOutput {
            id: output.id,
            name: output.name.clone(),
            output_type: match output.output_type {
                registry::MixerOutputType::Cue => PmxOutputType::Cue as i32,
                registry::MixerOutputType::Main => PmxOutputType::Main as i32,
            },
            left_port_path: match &output.pipewire_ports {
                PipewirePorts::None => None,
                PipewirePorts::Mono(left) => Some(left.clone()),
                PipewirePorts::Stereo(left, _) => Some(left.clone()),
            },
            right_port_path: match &output.pipewire_ports {
                PipewirePorts::None => None,
                PipewirePorts::Mono(left) => Some(left.clone()),
                PipewirePorts::Stereo(_, right) => Some(right.clone()),
            },
        }))
    }

    async fn register_output_stage(
        &self,
        request: Request<RegisterOutputStageRequest>,
    ) -> Result<Response<PmxOutputStage>, Status> {
        let inner = request.into_inner();
        let mut registry = self.registry.write().await;
        registry.register_output_stage(PmxOutputStage {
            id: 0,
            name: inner.name.clone(),
            left_channel_strip_id: inner.left_channel_strip_id,
            right_channel_strip_id: inner.right_channel_strip_id,
            cross_fader_plugin_id: inner.cross_fader_plugin_id,
        });
        Ok(Response::new(PmxOutputStage {
            id: 0,
            name: inner.name,
            left_channel_strip_id: inner.left_channel_strip_id,
            right_channel_strip_id: inner.right_channel_strip_id,
            cross_fader_plugin_id: inner.cross_fader_plugin_id,
        }))
    }

    async fn list_output_stages(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<ListOutputStagesReply>, Status> {
        let registry = self.registry.read().await;
        let output_stages = registry.get_all_output_stages();
        Ok(Response::new(ListOutputStagesReply {
            output_stages: output_stages
                .iter()
                .map(|o| PmxOutputStage {
                    id: o.id,
                    name: o.name.clone(),
                    left_channel_strip_id: o.left_channel_strip_id,
                    right_channel_strip_id: o.right_channel_strip_id,
                    cross_fader_plugin_id: o.cross_fader_plugin_id,
                })
                .collect(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    let data_paths = fr_pmx_config_lib::read_data_file_paths();
    let service_address = fr_pmx_config_lib::read_service_urls()
        .pmx_registry_url
        .replace("http://", "");
    let addr = service_address.parse().unwrap();

    let initial_inputs = file_reader::read_inputs_file(&data_paths.pmx_registry_data_file).await;
    let initial_outputs =
        file_reader::read_outputs_file(&data_paths.pmx_registry_output_data_file).await;
    let (outputs_sender, outputs_receiver) = tokio::sync::mpsc::unbounded_channel();
    let service = PmxRegistryService::new(initial_inputs, initial_outputs, sender, outputs_sender);
    let server = Server::builder()
        .add_service(PmxRegistryServer::new(service))
        .serve(addr);

    let file_writer =
        file_writer::run_input_file_writer(receiver, &data_paths.pmx_registry_data_file);

    let outputs_file_writer = file_writer::run_output_file_writer(
        outputs_receiver,
        &data_paths.pmx_registry_output_data_file,
    );

    tokio::select! {
        _ = server => {Ok(())}
        _ = file_writer => {Ok(())}
        _ = outputs_file_writer => {Ok(())}
    }
}
