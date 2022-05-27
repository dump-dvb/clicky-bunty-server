use super::{Role, ServiceResponse, User, UserConnection};

use pbkdf2::{
    password_hash::{
        Encoding, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Pbkdf2,
};

use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct RegisterUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub name: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct ModifyUserRequest {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<Role>,
}

#[derive(Deserialize, Serialize)]
pub struct DeleteUserRequest {
    pub id: String,
}

#[derive(Deserialize, Serialize)]
pub struct RequestUserSession {
    pub id: String,
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

pub async fn create_user(connection: &mut UserConnection, request: RegisterUserRequest) {
    let email_regex = Regex::new(
        r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})",
    )
    .unwrap();

    if !email_regex.is_match(&request.email) {
        return;
    }

    let password_hash = hash_password(&request.password);

    let role = if connection.database.lock().unwrap().first_user().await {
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

    let result = connection.database.lock().unwrap().create_user(&user).await;

    let serialized = serde_json::to_string(&ServiceResponse { success: true }).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized));
}

pub async fn login(connection: &mut UserConnection, request: LoginRequest) {
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
                    let serialized =
                        serde_json::to_string(&ServiceResponse { success: true }).unwrap();
                    connection
                        .socket
                        .write_message(tungstenite::Message::Text(serialized));
                    return;
                }
                _ => {}
            }
        }
        _ => {}
    }
    let serialized = serde_json::to_string(&ServiceResponse { success: false }).unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized));
}

pub async fn get_session(connection: &mut UserConnection) {
    let serialized = serde_json::to_string(&RequestUserSession {
        id: connection.user.as_ref().unwrap().id.to_string(),
    })
    .unwrap();
    connection
        .socket
        .write_message(tungstenite::Message::Text(serialized));
}

pub async fn delete_user(connection: &mut UserConnection, delete_request: DeleteUserRequest) {
    let user_id = connection.user.as_ref().unwrap().id.to_string();

    if connection
        .database
        .lock()
        .unwrap()
        .is_administrator(&user_id)
        .await
        || user_id == delete_request.id
    {
        connection
            .database
            .lock()
            .unwrap()
            .delete_user(&delete_request.id)
            .await;
    } else {
        let serialized = serde_json::to_string(&ServiceResponse { success: false }).unwrap();
        connection
            .socket
            .write_message(tungstenite::Message::Text(serialized));
    }
}

pub async fn modify_user(connection: &mut UserConnection, modify_request: ModifyUserRequest) {
    let user_struct_result = connection
        .database
        .lock()
        .unwrap()
        .query_user_by_id(&modify_request.id)
        .await;
    if user_struct_result.is_none() {
        return;
    }

    let user_struct = user_struct_result.unwrap();
    let user_id = connection.user.as_ref().unwrap().id.to_string();

    if connection
        .database
        .lock()
        .unwrap()
        .is_administrator(&user_id)
        .await
        || user_id == modify_request.id
    {
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
            id: Uuid::parse_str(&modify_request.id).unwrap(),
            name: modify_request.name.clone().unwrap_or(user_struct.name),
            email: modify_request.email.clone().unwrap_or(user_struct.email),
            password: hashed_password,
            role: modify_request.role.clone().unwrap_or(user_struct.role),
        });
    } else {
        let serialized = serde_json::to_string(&ServiceResponse { success: false }).unwrap();
        connection
            .socket
            .write_message(tungstenite::Message::Text(serialized));
    }
}
