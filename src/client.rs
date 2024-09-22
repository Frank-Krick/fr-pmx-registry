use clap::{Parser, Subcommand};
use pmx::{
    input::PmxInputType, pmx_registry_client::PmxRegistryClient, ByIdRequest, EmptyRequest,
    UpdateInputNameRequest, UpdateInputPortAssignmentsRequest,
};
use std::result::Result;
use tonic::Request;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Arguments {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    ListInputs {},
    GetInput {
        #[arg(short, long)]
        id: u32,
    },
    UpdateInputName {
        #[arg(short, long)]
        id: u32,
        #[arg(short, long)]
        name: String,
    },
    AssignMonoPort {
        #[arg(short, long)]
        id: u32,
        #[arg(short, long)]
        path: String,
    },
    AssignStereoPort {
        #[arg(short, long)]
        id: u32,
        #[arg(short, long)]
        left_path: String,
        #[arg(short, long)]
        right_path: String,
    },
    RemovePort {
        #[arg(short, long)]
        id: u32,
    },
    ListPlugins {},
    ListChannelStrips {},
    ListLoopers {},
    ListOutputs {},
}

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_arguments = Arguments::parse();

    if let Some(command) = cli_arguments.command {
        match command {
            Commands::ListChannelStrips {} => {
                let mut client = PmxRegistryClient::connect("http://127.0.0.1:50001").await?;
                let request = Request::new(EmptyRequest {});
                let response = client.list_channel_strips(request).await?;
                println!("{response:#?}");
            }
            Commands::ListInputs {} => {
                let mut client = PmxRegistryClient::connect("http://127.0.0.1:50001").await?;
                let request = Request::new(EmptyRequest {});
                let response = client.list_inputs(request).await?;
                println!("{response:#?}");
            }
            Commands::GetInput { id } => {
                let mut client = PmxRegistryClient::connect("http://127.0.0.1:50001").await?;
                let request = Request::new(ByIdRequest { id });
                let response = client.get_input(request).await?;
                println!("{response:#?}");
            }
            Commands::UpdateInputName { name, id } => {
                let mut client = PmxRegistryClient::connect("http://127.0.0.1:50001").await?;
                let request = Request::new(UpdateInputNameRequest { name, id });
                let response = client.update_input_name(request).await?;
                println!("{response:#?}");
            }
            Commands::RemovePort { id } => {
                let mut client = PmxRegistryClient::connect("http://127.0.0.1:50001").await?;
                let request = Request::new(UpdateInputPortAssignmentsRequest {
                    id,
                    input_type: PmxInputType::None as i32,
                    left_port_path: None,
                    right_port_path: None,
                });
                let response = client.update_input_port_assignments(request).await?;
                println!("{response:#?}");
            }
            Commands::AssignMonoPort { id, path } => {
                let mut client = PmxRegistryClient::connect("http://127.0.0.1:50001").await?;
                let request = Request::new(UpdateInputPortAssignmentsRequest {
                    id,
                    input_type: PmxInputType::MonoInput as i32,
                    left_port_path: Some(path),
                    right_port_path: None,
                });
                let response = client.update_input_port_assignments(request).await?;
                println!("{response:#?}");
            }
            Commands::AssignStereoPort {
                id,
                left_path,
                right_path,
            } => {
                let mut client = PmxRegistryClient::connect("http://127.0.0.1:50001").await?;
                let request = Request::new(UpdateInputPortAssignmentsRequest {
                    id,
                    input_type: PmxInputType::StereoInput as i32,
                    left_port_path: Some(left_path),
                    right_port_path: Some(right_path),
                });
                let response = client.update_input_port_assignments(request).await?;
                println!("{response:#?}");
            }
            Commands::ListPlugins {} => {
                let mut client = PmxRegistryClient::connect("http://127.0.0.1:50001").await?;
                let request = Request::new(EmptyRequest {});
                let response = client.list_plugins(request).await?;
                println!("{response:#?}");
            }
            Commands::ListLoopers {} => {
                let mut client = PmxRegistryClient::connect("http://127.0.0.1:50001").await?;
                let request = Request::new(EmptyRequest {});
                let response = client.list_loopers(request).await?;
                println!("{response:#?}");
            }
            Commands::ListOutputs {} => {
                let mut client = PmxRegistryClient::connect("http://127.0.0.1:50001").await?;
                let request = Request::new(EmptyRequest {});
                let response = client.list_outputs(request).await?;
                println!("{response:#?}");
            }
        }
    }

    Ok(())
}
