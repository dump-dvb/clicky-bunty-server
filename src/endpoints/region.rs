use super::{Region, ServiceResponse, UserConnection};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RegionRequest {
    pub name: String,
    pub transport_company: String,
    pub frequency: u64,
    pub protocol: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteRegion {
    id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ModifyRegionRequest {
    id: u32,
    pub name: Option<String>,
    pub transport_company: Option<String>,
    pub frequency: Option<u64>,
    pub protocol: Option<String>,
}

pub async fn create_region(connection: &mut UserConnection, request: RegionRequest) {
    let result = connection
        .database
        .lock()
        .unwrap()
        .create_region(&Region {
            id: 0,
            name: request.name,
            transport_company: request.transport_company,
            frequency: request.frequency,
            protocol: request.protocol,
        })
        .await;
    let serialized = serde_json::to_string(&ServiceResponse { success: result }).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized)).unwrap();
}

pub async fn modify_region(connection: &mut UserConnection, request: ModifyRegionRequest) {
    let result_region = connection
        .database
        .lock()
        .unwrap()
        .query_region(&request.id)
        .await;

    if result_region.is_none() {
        let serialized = serde_json::to_string(&ServiceResponse { success: false }).unwrap();
        connection
            .socket
            .write_message(tungstenite::Message::Text(serialized)).unwrap();

        return;
    }

    let region = result_region.unwrap();

    let result = connection
        .database
        .lock()
        .unwrap()
        .update_region(&Region {
            id: 0,
            name: request.name.unwrap_or(region.name),
            transport_company: request
                .transport_company
                .unwrap_or(region.transport_company),
            frequency: request.frequency.unwrap_or(region.frequency),
            protocol: request.protocol.unwrap_or(region.protocol),
        })
        .await;
    let serialized = serde_json::to_string(&ServiceResponse { success: result }).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized)).unwrap();
}

pub async fn delete_region(connection: &mut UserConnection, request: DeleteRegion) {
    let result = connection
        .database
        .lock()
        .unwrap()
        .delete_region(&request.id)
        .await;

    let serialized = serde_json::to_string(&ServiceResponse { success: result }).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized)).unwrap();
}

pub async fn list_regions(connection: &mut UserConnection) {
    let data = connection.database.lock().unwrap().list_regions().await;

    let serialized = serde_json::to_string(&data).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized)).unwrap();
}
