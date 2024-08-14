use registry::MixerInput;
use std::result::Result;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};

use pmx::input::{PmxInput, PmxInputType};
use pmx::pmx_registry_server::{PmxRegistry, PmxRegistryServer};
use pmx::{
    ByIdRequest, EmptyRequest, ListInputsReply, UpdateInputNameRequest,
    UpdateInputPortAssignmentsRequest,
};

use crate::registry::{PipewirePorts, Registry};

pub mod pmx {
    tonic::include_proto!("pmx");

    pub mod input {
        tonic::include_proto!("pmx.input");
    }
}

mod registry;

#[derive(Debug)]
pub struct PmxRegistryService {
    registry: RwLock<Registry>,
}

impl PmxRegistryService {
    fn new() -> Self {
        PmxRegistryService {
            registry: RwLock::new(Registry::new()),
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
        }
    }
}

#[tonic::async_trait]
impl PmxRegistry for PmxRegistryService {
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50001".parse().unwrap();
    let service = PmxRegistryService::new();
    Server::builder()
        .add_service(PmxRegistryServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}
