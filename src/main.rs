use crate::cosmos::NodeConfig;
use futures::{future, StreamExt};

mod cosmos;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cosmos_node_config = NodeConfig::LOCAL;
    let cosmos_client = cosmos::Client::new(cosmos_node_config)
        .await
        .expect("connecting to node's websocket should work")
        .run()
        .expect("subscribing to websocket should work");

    // Grab 5 NewBlock events
    let ev_count = 5 as usize;

    let subs = cosmos_client.subscribe_to_blocks().await.unwrap();

    let _ = subs.take(ev_count).for_each(|event| {
        let ev = event.unwrap();
        println!("{}", serde_json::to_string_pretty(&ev).unwrap());
        future::ready(())
    }).await;

    cosmos_client.close().unwrap();
    Ok(())
}
