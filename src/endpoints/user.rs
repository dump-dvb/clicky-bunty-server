use super::{Role, ServiceResponse, User, UserConnection};

use pbkdf2::{
    password_hash::{Encoding, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};

use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct RegisterUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginRequest {
    pub name: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ModifyUserRequest {
    pub id: Uuid,
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<Role>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UuidRequest {
    pub id: Uuid,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UuidResponse {
    pub id: Uuid,
    pub success: bool
}


fn hash_password(password: &String) -> String {
    let default_salt_path = String::from("/run/secrets/clicky_bunty_salt");
    let salt_path = std::env::var("SALT_PATH").unwrap_or(default_salt_path);
    let salt = SaltString::b64_encode(std::fs::read(salt_path).unwrap().as_slice()).unwrap();

    let password_hash = Pbkdf2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    PasswordHash::new(&password_hash).unwrap().to_string()
}

pub fn create_user(connection: &mut UserConnection, request: RegisterUserRequest) {
    if connection
        .database
        .lock()
        .unwrap()
        .check_user_exists(&request.name)
    {
        let serialized = serde_json::to_string(&ServiceResponse { success: false, message: Some("name already taken".to_string()) }).unwrap();
        connection
            .socket
            .write_message(tungstenite::Message::Text(serialized))
            .unwrap();

        return;
    }

    let email_regex = Regex::new(
        r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})",
    )
    .unwrap();

    if !email_regex.is_match(&request.email) {
        return;
    }

    let password_hash = hash_password(&request.password);

    let role = if connection.database.lock().unwrap().first_user() {
        Role::Administrator
    } else {
        Role::User
    };

    let user = User {
        id: Uuid::new_v4(),
        name: request.name,
        email: request.email,
        password: password_hash,
        role: role,
    };

    let result = connection.database.lock().unwrap().create_user(&user);

    let serialized = serde_json::to_string(&UuidResponse { id: user.id, success: result }).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized))
        .unwrap();
}

pub fn login(connection: &mut UserConnection, request: LoginRequest) {
    match connection
        .database
        .lock()
        .unwrap()
        .query_user(&request.name)
    {
        Some(user) => {
            println!("Found unter with this name !");
            let password_hash = PasswordHash::parse(&user.password, Encoding::B64).unwrap();
            match Pbkdf2.verify_password(request.password.as_bytes(), &password_hash) {
                Ok(_) => {
                    connection.user = Some(user.clone());
                    let serialized =
                        serde_json::to_string(&UuidResponse { id: user.id, success: true }).unwrap();
                    connection
                        .socket
                        .write_message(tungstenite::Message::Text(serialized))
                        .unwrap();
                    return;
                }
                _ => {
                    println!("Password does not match");
                }
            }
        }
        _ => {
            println!("No user found ! {}", &request.name);
        }
    }
    let serialized = serde_json::to_string(&ServiceResponse { success: false, message: Some("could not login user name or password wrong".to_string()) }).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized))
        .unwrap();
}

pub fn get_session(connection: &mut UserConnection) {
    let serialized = serde_json::to_string(&UuidRequest {
        id: connection.user.as_ref().unwrap().id,
    }).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized))
        .unwrap();
}

pub fn delete_user(connection: &mut UserConnection, delete_request: UuidRequest) {
    let user_id = connection.user.as_ref().unwrap().id;

    if connection
        .database
        .lock()
        .unwrap()
        .is_administrator(&user_id)
        || user_id == delete_request.id
    {
        connection
            .database
            .lock()
            .unwrap()
            .delete_user(&delete_request.id);
    } else {
        let serialized = serde_json::to_string(&ServiceResponse { success: false, message: Some("you are not administrator or this user".to_string()) }).unwrap();
        connection
            .socket
            .write_message(tungstenite::Message::Text(serialized))
            .unwrap();
    }
}

pub fn modify_user(connection: &mut UserConnection, modify_request: ModifyUserRequest) {
    let user_struct_result = connection
        .database
        .lock()
        .unwrap()
        .query_user_by_id(&modify_request.id);

    if user_struct_result.is_none() {
        return;
    }

    let user_struct = user_struct_result.unwrap();
    let user_id = connection.user.as_ref().unwrap().id;
    let admin = connection
        .database
        .lock()
        .unwrap()
        .is_administrator(&user_id);

    if admin || user_id == modify_request.id {
        if !admin && modify_request.role.is_some() {
            // only admins can change the role of a suer
            let serialized = serde_json::to_string(&ServiceResponse { success: false, message: Some("you are not administrator or this user".to_string()) }).unwrap();
            connection
                .socket
                .write_message(tungstenite::Message::Text(serialized))
                .unwrap();

            return;
        }

        let hashed_password: String;
        match &modify_request.password {
            Some(password) => {
                hashed_password = hash_password(&password);
            }
            _ => {
                hashed_password = user_struct.password;
            }
        }

        connection.database.lock().unwrap().update_user(&User {
            id: modify_request.id,
            name: modify_request.name.clone().unwrap_or(user_struct.name),
            email: modify_request.email.clone().unwrap_or(user_struct.email),
            password: hashed_password,
            role: modify_request.role.clone().unwrap_or(user_struct.role),
        });
    } else {
        let serialized = serde_json::to_string(&ServiceResponse { success: false, message: Some("you are not administrator or this user".to_string()) }).unwrap();
        connection
            .socket
            .write_message(tungstenite::Message::Text(serialized))
            .unwrap();
    }
}

pub fn list_users(connection: &mut UserConnection) {
    println!("current user: {:?}", &connection.user);
    if connection.user.as_ref().unwrap().role == Role::Administrator {
            let users = connection.database.lock().unwrap().list_users();

            let serialized = serde_json::to_string(&users).unwrap();
            connection
                .socket
                .write_message(tungstenite::Message::Text(serialized))
                .unwrap();

    } else {
        let serialized = serde_json::to_string(&ServiceResponse { success: false, message: Some("you are not administrator".to_string()) }).unwrap();
        connection
            .socket
            .write_message(tungstenite::Message::Text(serialized))
            .unwrap();
    }
}

