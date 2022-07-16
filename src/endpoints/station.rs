use super::{UuidRequest, ServiceResponse, Station, UserConnection};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateStationRequest {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub region: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ListStationsRequest {
    pub desired_owner: Option<String>,
    pub desired_region: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ModifyStation {
    pub id: Uuid,
    pub name: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub region: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApproveStation {
    pub id: Uuid,
    pub approved: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UuidResponse {
    pub id: Uuid,
    pub success: bool,
}

fn write_result(response: bool, connection: &mut UserConnection) {
    let serialized = serde_json::to_string(&ServiceResponse {
        success: response,
        message: None
    }).unwrap();

    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized))
        .unwrap();
}


fn owns_station(connection: &mut UserConnection, station_id: &Uuid) -> bool {
    let result_station = connection
        .database
        .lock()
        .unwrap()
        .query_station(&station_id);

    if result_station.is_none() {
        return false;
    }

    let station = result_station.unwrap();

    station.owner == connection.user.as_ref().unwrap().id
}

pub fn create_station(connection: &mut UserConnection, request: CreateStationRequest) {
    if connection
        .database
        .lock()
        .unwrap()
        .check_region_exists(request.region)
        && connection.user.is_some()
    {
        let random_token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let station = Station {
            token: Some(random_token),
            id: Uuid::new_v4(),
            name: request.name,
            lat: request.lat,
            lon: request.lon,
            region: request.region,
            owner: connection.user.as_ref().unwrap().id,
            approved: false,
        };

        let result = connection.database.lock().unwrap().create_station(&station);
        let serialized = serde_json::to_string(&UuidResponse {
            success: true,
            id: station.id
        }).unwrap();

        connection
            .socket
            .write_message(tungstenite::Message::Text(serialized))
            .unwrap();

    } else {
        write_result(false, connection);
    }
}

pub fn list_stations(connection: &mut UserConnection, request: ListStationsRequest) {
    let data = connection
        .database
        .lock()
        .unwrap()
        .list_stations(request.desired_owner, request.desired_region);
    
    let serialized = serde_json::to_string(&data).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized))
        .unwrap();
}

pub fn delete_station(connection: &mut UserConnection, request: UuidRequest) {
    let mut result_query = false;
    if connection.user.as_ref().unwrap().is_admin() || owns_station(connection, &request.id) {
        result_query = connection
            .database
            .lock()
            .unwrap()
            .delete_station(&request.id);
    }

    write_result(result_query, connection);
}

pub fn modify_station(connection: &mut UserConnection, request: ModifyStation) {
    let result_station = connection
        .database
        .lock()
        .unwrap()
        .query_station(&request.id);

    if result_station.as_ref().is_none() {
        write_result(false, connection);
        return;
    }

    let station = result_station.unwrap();
    if connection.user.as_ref().unwrap().is_admin() || owns_station(connection, &request.id) {
        let response = connection
            .database
            .lock()
            .unwrap()
            .update_station(&Station {
                id: request.id,
                approved: connection.user.as_ref().unwrap().is_admin(),
                name: request.name.as_ref().unwrap_or(&station.name).to_string(),
                lat: request.lat.unwrap_or(station.lat),
                lon: request.lon.unwrap_or(station.lon),
                region: request.region.unwrap_or(station.region),
                token: None,
                owner: station.owner,
            });
        write_result(response, connection);
    } else {
        write_result(false, connection);
    }
}

pub fn approve_station(connection: &mut UserConnection, request: ApproveStation) {
    if connection.user.as_ref().unwrap().is_admin() {
        let response = connection
            .database
            .lock()
            .unwrap()
            .set_approved(&request.id, request.approved);
        write_result(response, connection);
    } else {
        write_result(false, connection);
    }
}

pub fn generate_token(connection: &mut UserConnection, request: UuidRequest) {
    if connection.user.as_ref().unwrap().is_admin() || owns_station(connection, &request.id) {
        let random_token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let response = connection
            .database
            .lock()
            .unwrap()
            .set_token(&request.id, &random_token);

        write_result(response, connection);
    } else {
        write_result(false, connection);
    }
}
