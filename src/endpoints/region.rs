use super::IdentifierRequest;
use super::{Region, ServiceResponse, UserConnection};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RegionRequest {
    pub name: String,
    pub transport_company: String,
    pub frequency: u64,
    pub protocol: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModifyRegionRequest {
    id: u32,
    pub name: Option<String>,
    pub transport_company: Option<String>,
    pub frequency: Option<u64>,
    pub protocol: Option<String>,
}

fn admin(connection: &mut UserConnection) -> bool {
    connection.user.as_ref().unwrap().is_admin()
}

fn write_error(connection: &mut UserConnection) {
    let serialized = serde_json::to_string(&ServiceResponse { success: false }).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized))
        .unwrap();
}

pub fn create_region(connection: &mut UserConnection, request: RegionRequest) {
    println!("message: {}", admin(connection));
    if !admin(connection) {
        write_error(connection);
        return;
    }

    let result = connection.database.lock().unwrap().create_region(&Region {
        id: 0,
        name: request.name,
        transport_company: request.transport_company,
        frequency: request.frequency,
        protocol: request.protocol,
    });


    let serialized = serde_json::to_string(&ServiceResponse { success: result }).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized))
        .unwrap();
}

pub fn modify_region(connection: &mut UserConnection, request: ModifyRegionRequest) {
    if !admin(connection) {
        write_error(connection);
        return;
    }

    let result_region = connection
        .database
        .lock()
        .unwrap()
        .query_region(&request.id);

    if result_region.is_none() {
        write_error(connection);
        return;
    }

    let region = result_region.unwrap();

    let result = connection.database.lock().unwrap().update_region(&Region {
        id: 0,
        name: request.name.unwrap_or(region.name),
        transport_company: request
            .transport_company
            .unwrap_or(region.transport_company),
        frequency: request.frequency.unwrap_or(region.frequency),
        protocol: request.protocol.unwrap_or(region.protocol),
    });
    let serialized = serde_json::to_string(&ServiceResponse { success: result }).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized))
        .unwrap();
}

pub fn delete_region(connection: &mut UserConnection, request: IdentifierRequest) {
    if !admin(connection) {
        write_error(connection);
        return;
    }

    let result = connection
        .database
        .lock()
        .unwrap()
        .delete_region(&request.id);

    let serialized = serde_json::to_string(&ServiceResponse { success: result }).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized))
        .unwrap();
}

pub fn list_regions(connection: &mut UserConnection) {
    let data = connection.database.lock().unwrap().list_regions();

    let serialized = serde_json::to_string(&data).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized))
        .unwrap();
}
