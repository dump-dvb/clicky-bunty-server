mod database;
mod endpoints;
mod structs;

pub use database::{DataBaseConnection, Region, Role, Station, User};
use endpoints::{
    approve_station, create_region, create_station, create_user, list_users, delete_region, delete_station,
    delete_user, generate_token, get_session, list_regions, list_stations, login, modify_region,
    modify_station, modify_user, Body,
};
use structs::Args;

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
    body: Option<Body>,
}

#[derive(Serialize)]
pub struct ServiceResponse {
    success: bool,
}

pub struct UserConnection {
    database: Arc<Mutex<DataBaseConnection>>,
    socket: tungstenite::protocol::WebSocket<std::net::TcpStream>,
    user: Option<User>,
}

fn process_message(connection: &mut UserConnection, message: &tungstenite::protocol::Message) {
    let command: String;
    let body: Body;

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
                        serde_json::to_string(&ServiceResponse { success: false }).unwrap();
                    connection
                        .socket
                        .write_message(tungstenite::Message::Text(serialized))
                        .unwrap();

                    return;
                }
            }
            command = parsed.operation;
            match parsed.body {
                Some(body_found) => {
                    body = body_found;
                }
                _ => {
                    body = Body::Empty;
                }
            }
        }
        _ => {
            return;
        }
    }

    let authenticated = connection.user.is_some();

    match (command.as_str(), body, authenticated) {
        ("user/register", Body::Register(parsed_struct), false) => {
            create_user(connection, parsed_struct);
        }
        ("user/login", Body::Login(parsed_struct), false) => {
            login(connection, parsed_struct);
        }
        ("user/session", Body::Empty, true) => {
            get_session(connection);
        }
        ("user/delete", Body::UserIdentifier(parsed_struct), true) => {
            delete_user(connection, parsed_struct);
        }
        ("user/modify", Body::UserModify(parsed_struct), true) => {
            modify_user(connection, parsed_struct);
        }
        ("user/list", Body::Empty, true) => {
            list_users(connection);
        }
        ("station/create", Body::CreateStation(parsed_struct), true) => {
            create_station(connection, parsed_struct);
        }
        ("station/list", Body::ListStations(parsed_struct), _) => {
            list_stations(connection, parsed_struct);
        }
        ("station/delete", Body::Identifier(parsed_struct), true) => {
            delete_station(connection, parsed_struct);
        }
        ("station/modify", Body::ModifyStation(parsed_struct), true) => {
            modify_station(connection, parsed_struct);
        }
        ("station/approve", Body::ApproveStation(parsed_struct), true) => {
            approve_station(connection, parsed_struct);
        }
        ("station/generate_token", Body::Identifier(parsed_struct), true) => {
            generate_token(connection, parsed_struct);
        }
        ("region/create", Body::CreateRegion(parsed_struct), true) => {
            create_region(connection, parsed_struct);
        }
        ("region/delete", Body::Identifier(parsed_struct), true) => {
            delete_region(connection, parsed_struct);
        }
        ("region/modify", Body::ModifyRegion(parsed_struct), true) => {
            modify_region(connection, parsed_struct);
        }
        ("region/list", Body::Empty, _) => {
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
