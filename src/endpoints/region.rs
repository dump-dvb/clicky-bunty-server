
use super::{UserConnection, Region};

pub async fn list_regions(connection: &mut UserConnection) {
    let data = connection.database.lock().unwrap().list_regions().await;

    let serialized = serde_json::to_string(&data).unwrap();
    connection.socket.write_message(tungstenite::Message::Text(serialized));
}
