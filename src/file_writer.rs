use tokio::{fs::File, io::AsyncWriteExt};

use crate::registry::MixerInput;

pub async fn run_file_writer(
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<[MixerInput; 9]>,
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
