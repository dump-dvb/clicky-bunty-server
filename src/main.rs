mod database;
mod structs;

use database::{DataBaseConnection, Region, Station, User};
use structs::Args;

use chrono::{DateTime, Utc};
use clap::Parser;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;
use rand::{distributions::Alphanumeric, Rng};
use tokio;
use tonic::{transport::Server, Request, Response, Status};
use tungstenite::accept;
use pbkdf2::{
    password_hash::{
        rand_core::OsRng, Encoding, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Pbkdf2,
};
use std::net::TcpListener;

/*  TODO:
 *  - admin user (first user creates)
 *  - making users to admins
 *  - region create
 *  - modifing stations, regions, users
 *  - deleting stations, regions, users
 */

#[derive(Serialize)]
struct ServiceResponse {
    success: bool,
}

#[derive(Deserialize, Serialize)]
struct RegisterUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
struct LoginRequest {
    pub name: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
struct CreateStationRequest {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub region: u32,
}

#[derive(Deserialize, Serialize)]
struct ListStationsRequest {
    pub owner: Option<String>,
    pub region: Option<u32>,
}

#[derive(Deserialize, Serialize)]
enum Body {
    Empty,
    Register(RegisterUserRequest),
    Login(LoginRequest),
    CreateStation(CreateStationRequest),
    ListStations(ListStationsRequest),
}

#[derive(Deserialize, Serialize)]
struct MessageTemplate {
    operation: String,
    body: Body,
}

pub struct UserConnection {
    database: Arc<Mutex<DataBaseConnection>>,
    socket: tungstenite::protocol::WebSocket<std::net::TcpStream>,
    user: Option<User>,
}

async fn create_user(connection: &mut UserConnection, request: RegisterUserRequest) {
    let email_regex = Regex::new(
        r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})",
    )
    .unwrap();

    if !email_regex.is_match(&request.email) {
        return;
    }

    let default_salt_path = String::from("/run/secrets/clicky_bunty_salt");
    let salt_path = std::env::var("SALT_PATH").unwrap_or(default_salt_path);
    let salt = SaltString::b64_encode(std::fs::read(salt_path).unwrap().as_slice()).unwrap();

    let password_hash = Pbkdf2
        .hash_password(request.password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    let parsed_hash = PasswordHash::new(&password_hash).unwrap();
    
    let role = if connection.database.lock().unwrap().first_user().await {0} else {6};

    let user = User {
        id: Uuid::new_v4(),
        name: request.name,
        email: request.email,
        password: parsed_hash.to_string(),
        role: role
    };

    let result = connection.database.lock().unwrap().create_user(&user).await;

    let serialized = serde_json::to_string(&ServiceResponse { success: true }).unwrap();
    connection.socket.write_message(tungstenite::Message::Text(serialized));
}

async fn login(connection: &mut UserConnection, request: LoginRequest) {
    match connection
        .database
        .lock()
        .unwrap()
        .query_user(&request.name)
        .await
    {
        Some(user) => {
            let default_salt_path = String::from("/run/secrets/clicky_bunty_salt");
            let salt_path = std::env::var("SALT_PATH").unwrap_or(default_salt_path);
            let salt =
                SaltString::b64_encode(std::fs::read(salt_path).unwrap().as_slice()).unwrap();
            let password_hash = PasswordHash::parse(&user.password, Encoding::B64).unwrap();
            match Pbkdf2.verify_password(request.password.as_bytes(), &password_hash) {
                Ok(_) => {
                    connection.user = Some(user);
                    let serialized = serde_json::to_string(&ServiceResponse { success: true }).unwrap();
                    connection.socket.write_message(tungstenite::Message::Text(serialized));
                    return;
                }
                _ => {}
            }
        }
        _ => { }
    }
    let serialized = serde_json::to_string(&ServiceResponse { success: false }).unwrap();
    connection.socket.write_message(tungstenite::Message::Text(serialized));
}

async fn create_station(connection: &mut UserConnection, request: CreateStationRequest) {
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
        connection.socket.write_message(tungstenite::Message::Text(serialized));
    } else {
        let serialized = serde_json::to_string(&ServiceResponse { success: false }).unwrap();
        connection.socket.write_message(tungstenite::Message::Text(serialized));

    }
}

async fn list_stations(connection: &mut UserConnection, request: ListStationsRequest) {
    let data= connection.database.lock().unwrap().list_stations(None, None).await;

    let serialized = serde_json::to_string(&data).unwrap();
    connection.socket.write_message(tungstenite::Message::Text(serialized));
}


async fn list_regions(connection: &mut UserConnection) {
    let data = connection.database.lock().unwrap().list_regions().await;

    let serialized = serde_json::to_string(&data).unwrap();
    connection.socket.write_message(tungstenite::Message::Text(serialized));
}


async fn process_message(
    connection: &mut UserConnection,
    message: &tungstenite::protocol::Message,
) {
    let command: String;
    let body: Body;

    match message {
        tungstenite::protocol::Message::Text(text) => {
            let parsed: MessageTemplate = serde_json::from_str(&text).unwrap(); //TODO:
            command = parsed.operation;
            body = parsed.body;
        }
        _ => {
            return;
        }
    }
    match (command.as_str(), body) {
        ("user/register", Body::Register(parsed_struct)) => {
            create_user(connection, parsed_struct);
        }
        ("user/login", Body::Login(parsed_struct)) => {
            login(connection, parsed_struct);
        }
        ("user/create", Body::CreateStation(parsed_struct)) => {
            create_station(connection, parsed_struct);
        }
        ("region/list", Body::Empty) => {
            list_regions(connection);
        }
        ("station/list", Body::ListStations(parsed_struct)) => {
            list_stations(connection, parsed_struct);
        }
        (&_, _) => {}
    }
}

async fn listen(mut connection: UserConnection) {
    loop {
        match connection.socket.read_message() {
            Ok(message) => {
                process_message(&mut connection, &message).await;
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let host = args.host.as_str();
    let port = args.port;
    let current_run = Arc::new(Mutex::new(DataBaseConnection::new().await));

    println!("Listening on: {}:{}", host, port);
    println!("Opening Websocket Sever ...");
    let server = TcpListener::bind(format!("{}:{}", host, port)).unwrap();
    for stream in server.incoming() {
        match accept(stream.unwrap()) {
            Ok(websocket) => {
                listen(UserConnection {
                    database: current_run.clone(),
                    socket: websocket,
                    user: None,
                });
            }
            _ => {}
        };
    }
}
