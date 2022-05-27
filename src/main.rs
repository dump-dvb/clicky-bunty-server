mod database;
mod structs;
mod endpoints;

pub use database::{DataBaseConnection, Region, Station, User, Role};
use structs::Args;
use endpoints::{
    create_user, 
    login, 
    get_session,
    delete_user,
    modify_user,
    create_station, 
    list_stations, 
    delete_station,
    modify_station,
    approve_station,
    generate_token,
    list_regions, 
    Body
};


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
use tokio;
use tonic::{transport::Server, Request, Response, Status};
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
    body: Body,
}

pub struct UserConnection {
    database: Arc<Mutex<DataBaseConnection>>,
    socket: tungstenite::protocol::WebSocket<std::net::TcpStream>,
    user: Option<User>,
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
    
    let authenticated = connection.user.is_some();

    match (command.as_str(), body, authenticated) {
        ("user/register", Body::Register(parsed_struct), false) => {
            create_user(connection, parsed_struct).await;
        }
        ("user/login", Body::Login(parsed_struct), false) => {
            login(connection, parsed_struct).await;
        },
        ("user/session", Body::Empty, true) => {
            get_session(connection).await;
        },
        ("user/delete", Body::DeleteUser(parsed_struct), true) => {
            delete_user(connection, parsed_struct).await;
        },
        ("user/modify", Body::UserModify(parsed_struct), true) => {
            modify_user(connection, parsed_struct).await;
        },
        ("station/create", Body::CreateStation(parsed_struct), true) => {
            create_station(connection, parsed_struct).await;
        }
        ("station/list", Body::ListStations(parsed_struct), _) => {
            list_stations(connection, parsed_struct).await;
        }
        ("station/delete", Body::DeleteStation(parsed_struct), true) => {
            delete_station(connection, parsed_struct).await;
        },
        ("station/modify", Body::ModifyStation(parsed_struct), true) => {
            modify_station(connection, parsed_struct).await;
        },
        ("station/approve", Body::ApproveStation(parsed_struct), true) => {
            approve_station(connection, parsed_struct).await;
        },
        ("station/generate_token", Body::GenerateToken(parsed_struct), true) => {
            generate_token(connection, parsed_struct).await;
        },
        ("region/list", Body::Empty, _) => {
            list_regions(connection).await;
        }

        (&_, _, _) => {}
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
