#[macro_use]
extern crate diesel;
mod database;
mod endpoints;
mod structs;
mod schema;

pub use database::{DataBaseConnection, Region, Role, Station, User};
use endpoints::{
    approve_station, create_region, create_station, create_user, list_users, delete_region, delete_station,
    delete_user, generate_token, get_session, list_regions, list_stations, login, modify_region,
    modify_station, modify_user, ListStationsRequest, ApproveStation, CreateStationRequest, UuidRequest, RegisterUserRequest, LoginRequest, ModifyUserRequest, ModifyRegionRequest, RegionRequest, ModifyStation, IdentifierRequest
};
use structs::Args;

use serde::de::DeserializeOwned;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::accept;

use std::net::TcpListener;

/*  TODO:
 *  - admin user (first user creates)
 *  - making users to admins
 *  - region create
 *  - modifing stations, regions, users
 *  - deleting stations, regions, users
 */

#[derive(Deserialize, Serialize)]
struct MessageTemplate {
    operation: String,
    body: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct ServiceResponse {
    success: bool,
    message: Option<String>
}

pub struct UserConnection {
    database: Arc<Mutex<DataBaseConnection>>,
    socket: tungstenite::protocol::WebSocket<std::net::TcpStream>,
    user: Option<User>,
}

fn call_backend<T: DeserializeOwned>(data: serde_json::Value, function: Box<dyn Fn(&mut UserConnection, T)>, connection: &mut UserConnection) 
{
    match serde_json::value::from_value::<T>(data) {
        Ok(parsed_struct) => {
            function(connection, parsed_struct);
        }
        _ => {
            let serialized = serde_json::to_string(&ServiceResponse { success: false , message: Some(String::from("decoding failed"))}).unwrap();
            connection
                .socket
                .write_message(tungstenite::Message::Text(serialized))
                .unwrap();
        }
    }
}

fn process_message(connection: &mut UserConnection, message: &tungstenite::protocol::Message) {
    let command: String;
    let raw_body: Option<serde_json::Value>;

    match message {
        tungstenite::protocol::Message::Text(text) => {
            let parsed: MessageTemplate;
            match serde_json::from_str(&text) {
                Ok(data) => {
                    parsed = data;
                }
                Err(e) => {
                    println!("user send incorrect message {:?}", e);
                    let serialized =
                        serde_json::to_string(&ServiceResponse { success: false, message: Some(String::from("operation entry is missing")) }).unwrap();
                    connection
                        .socket
                        .write_message(tungstenite::Message::Text(serialized))
                        .unwrap();

                    return;
                }
            }
            command = parsed.operation;
            raw_body = parsed.body;
        }
        _ => {
            return;
        }
    }

    let authenticated = connection.user.is_some();

    println!("command: {}, body: {:?}, authenticated: {}", &command.as_str(), &raw_body, authenticated);

    match (command.as_str(), raw_body, authenticated) {
        ("user/register", Some(body), false) => {
            call_backend::<RegisterUserRequest>(body, Box::new(create_user), connection);
        }
        ("user/login", Some(body), false) => {
            call_backend::<LoginRequest>(body, Box::new(login), connection);
        }
        ("user/session", None, true) => {
            get_session(connection);
        }
        ("user/delete", Some(body), true) => {
            call_backend::<UuidRequest>(body, Box::new(delete_user), connection);
        }
        ("user/modify", Some(body), true) => {
            call_backend::<ModifyUserRequest>(body, Box::new(modify_user), connection);
        }
        ("user/list", None, true) => {
            list_users(connection);
        }
        ("station/create", Some(body), true) => {
            call_backend::<CreateStationRequest>(body, Box::new(create_station), connection);
        }
        ("station/list", Some(body), _) => {
            call_backend::<ListStationsRequest>(body, Box::new(list_stations), connection);
        }
        ("station/list", None, _) => {
            list_stations(connection, ListStationsRequest { desired_owner: None, desired_region: None});
        }
        ("station/delete", Some(body), true) => {
            call_backend::<UuidRequest>(body, Box::new(delete_station), connection);
        }
        ("station/modify", Some(body), true) => {
            call_backend::<ModifyStation>(body, Box::new(modify_station), connection);
        }
        ("station/approve", Some(body), true) => {
            call_backend::<ApproveStation>(body, Box::new(approve_station), connection);
        }
        ("station/generate_token", Some(body), true) => {
            call_backend::<UuidRequest>(body, Box::new(generate_token), connection);
        }
        ("region/create", Some(body), true) => {
            call_backend::<RegionRequest>(body, Box::new(create_region), connection);
        }
        ("region/delete", Some(body), true) => {
            call_backend::<IdentifierRequest>(body, Box::new(delete_region), connection);
        }
        ("region/modify", Some(body), true) => {
            call_backend::<ModifyRegionRequest>(body, Box::new(modify_region), connection);
        }
        ("region/list", None, _) => {
            list_regions(connection);
        }
        (&_, _, _) => {}
    }
}

fn listen(mut connection: UserConnection) {
    loop {
        match connection.socket.read_message() {
            Ok(message) => {
                println!("Received Message {:?} !", &message);
                process_message(&mut connection, &message);
            }
            _ => {}
        }
    }
}

//#[tokio::main]
fn main() {
    let args = Args::parse();

    let host = args.host.as_str();
    let port = args.port;
    let current_run = Arc::new(Mutex::new(DataBaseConnection::new()));

    println!("Listening on: {}:{}", host, port);
    println!("Opening Websocket Sever ...");
    let server = TcpListener::bind(format!("{}:{}", host, port)).unwrap();
    for stream in server.incoming() {
        let current_run_clone = current_run.clone();
        thread::spawn(move || {
            match accept(stream.unwrap()) {
                Ok(websocket) => {
                    println!("New Connection!");
                    listen(UserConnection {
                        database: current_run_clone,
                        socket: websocket,
                        user: None,
                    });
                }
                _ => {}
            };
        });
    }
}
