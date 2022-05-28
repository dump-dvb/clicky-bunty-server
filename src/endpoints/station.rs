use super::{ServiceResponse, Station, UserConnection};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateStationRequest {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub region: u32,
}

#[derive(Deserialize, Serialize)]
pub struct ListStationsRequest {
    pub owner: Option<String>,
    pub region: Option<u32>,
}

#[derive(Deserialize, Serialize)]
pub struct DeleteStation {
    pub id: u32,
}

#[derive(Deserialize, Serialize)]
pub struct ModifyStation {
    pub id: u32,
    pub name: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub region: Option<u32>,
}

#[derive(Deserialize, Serialize)]
pub struct ApproveStation {
    pub id: u32,
    pub approved: bool,
}

#[derive(Deserialize, Serialize)]
pub struct GenerateToken {
    pub id: u32,
}

async fn owns_station(connection: &mut UserConnection, station_id: &u32) -> bool {
    let result_station = connection
        .database
        .lock()
        .unwrap()
        .query_station(&station_id)
        .await;

    if result_station.is_none() {
        return false;
    }

    let station = result_station.unwrap();

    station.owner == connection.user.as_ref().unwrap().id
}

pub async fn create_station(connection: &mut UserConnection, request: CreateStationRequest) {
    if connection
        .database
        .lock()
        .unwrap()
        .check_region_exists(request.region)
        .await
        && connection.user.is_some()
    {
        let random_token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let station = Station {
            token: Some(random_token),
            id: 0,
            name: request.name,
            lat: request.lat,
            lon: request.lon,
            region: request.region,
            owner: connection.user.as_ref().unwrap().id,
            approved: false,
        };

        let result = connection
            .database
            .lock()
            .unwrap()
            .create_station(&station)
            .await;

        let serialized = serde_json::to_string(&ServiceResponse { success: result }).unwrap();
        connection
            .socket
            .write_message(tungstenite::Message::Text(serialized)).unwrap();
    } else {
        let serialized = serde_json::to_string(&ServiceResponse { success: false }).unwrap();
        connection
            .socket
            .write_message(tungstenite::Message::Text(serialized)).unwrap();
    }
}

pub async fn list_stations(connection: &mut UserConnection, request: ListStationsRequest) {
    let data = connection
        .database
        .lock()
        .unwrap()
        .list_stations(request.owner, request.region)
        .await;

    let serialized = serde_json::to_string(&data).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized)).unwrap();
}

pub async fn delete_station(connection: &mut UserConnection, request: DeleteStation) {
    let mut result_query = false;
    if connection.user.as_ref().unwrap().is_admin() || owns_station(connection, &request.id).await {
        result_query = connection
            .database
            .lock()
            .unwrap()
            .delete_station(&request.id)
            .await;
    }
    let serialized = serde_json::to_string(&ServiceResponse { success: result_query }).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized)).unwrap();
}

pub async fn modify_station(connection: &mut UserConnection, request: ModifyStation) {
    let result_station = connection
        .database
        .lock()
        .unwrap()
        .query_station(&request.id)
        .await;

    if result_station.as_ref().is_none() {
        return;
    }

    let station = result_station.unwrap();

    if connection.user.as_ref().unwrap().is_admin() || owns_station(connection, &request.id).await {
        connection
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
            })
            .await;
    }
}

pub async fn approve_station(connection: &mut UserConnection, request: ApproveStation) {
    if connection.user.as_ref().unwrap().is_admin() {
        connection
            .database
            .lock()
            .unwrap()
            .set_approved(&request.id, request.approved)
            .await;
    }
}

pub async fn generate_token(connection: &mut UserConnection, request: GenerateToken) {
    if connection.user.as_ref().unwrap().is_admin() || owns_station(connection, &request.id).await {
        let random_token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        connection
            .database
            .lock()
            .unwrap()
            .set_token(&request.id, &random_token)
            .await;
    }
}