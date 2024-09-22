use tokio::{fs::File, io::AsyncWriteExt};

use crate::registry::{MixerInput, MixerOutput};

pub async fn run_input_file_writer(
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<Vec<MixerInput>>,
    path: &String,
) {
    loop {
        let data = receiver.recv().await;
        if receiver.is_empty() {
            let data = serde_json::to_string_pretty(&data.unwrap()).unwrap();
            let mut file = File::create(path).await.unwrap();
            file.write_all(data.as_bytes()).await.unwrap();
        }
    }
}

pub async fn run_output_file_writer(
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<Vec<MixerOutput>>,
    path: &String,
) {
    loop {
        let data = receiver.recv().await;
        if receiver.is_empty() {
            let data = serde_json::to_string_pretty(&data.unwrap()).unwrap();
            let mut file = File::create(path).await.unwrap();
            file.write_all(data.as_bytes()).await.unwrap();
        }
    }
}
