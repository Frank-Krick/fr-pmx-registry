#[derive(Debug)]
pub struct MixerInput {
    pub name: String,
    pub pipewire_ports: PipewirePorts,
    pub id: u32,
}

#[derive(Debug, Clone)]
pub enum PipewirePorts {
    None,
    Mono(String),
    Stereo(String, String),
}

impl MixerInput {
    pub fn new(name: &str, pipewire_ports: PipewirePorts, id: u32) -> Self {
        MixerInput {
            id,
            name: String::from(name),
            pipewire_ports,
        }
    }
}

#[derive(Debug)]
pub struct Registry {
    inputs: [MixerInput; 9],
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

impl Registry {
    pub fn new() -> Self {
        Registry {
            inputs: [
                MixerInput::new("DSMPL", PipewirePorts::None, 1),
                MixerInput::new("DFire", PipewirePorts::None, 2),
                MixerInput::new("DEuro", PipewirePorts::None, 3),
                MixerInput::new("Prophet rev2", PipewirePorts::None, 4),
                MixerInput::new("SE02", PipewirePorts::None, 5),
                MixerInput::new("Torso S4", PipewirePorts::None, 6),
                MixerInput::new("opsix", PipewirePorts::None, 7),
                MixerInput::new("System 1m", PipewirePorts::None, 8),
                MixerInput::new("Cobalt 8m", PipewirePorts::None, 9),
            ],
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
            .iter()
            .enumerate()
            .find(|(_index, input)| input.id == id)
        {
            self.inputs[input.0].name = String::from(name);
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
            .iter()
            .enumerate()
            .find(|(_index, input)| input.id == id)
        {
            self.inputs[input.0].pipewire_ports = ports;
            Ok(())
        } else {
            Err(std::boxed::Box::new(NotFoundError {}))
        }
    }
}
