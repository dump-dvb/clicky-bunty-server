mod station;
mod user;
mod region;

pub use super::{UserConnection, User, Station, Region, Role};

pub use user::{
    create_user, 
    login, 
    get_session, 
    delete_user, 
    modify_user, 
    RegisterUserRequest, 
    LoginRequest,
    ModifyUserRequest,
    DeleteUserRequest,
};
pub use station::{
    create_station, 
    list_stations,
    delete_station,
    modify_station,
    approve_station,
    generate_token,
    CreateStationRequest, 
    ListStationsRequest,
    DeleteStation,
    ModifyStation,
    ApproveStation,
    GenerateToken
};

pub use region::{list_regions};

use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize)]
pub enum Body {
    Empty,
    Register(RegisterUserRequest),
    Login(LoginRequest),
    UserModify(ModifyUserRequest),
    DeleteUser(DeleteUserRequest),

    CreateStation(CreateStationRequest),
    ListStation(ListStationsRequest),
    DeleteStation(DeleteStation),
    ModifyStation(ModifyStation),
    ApproveStation(ApproveStation),
    GenerateToken(GenerateToken),

    ListStations(ListStationsRequest),
}

#[derive(Serialize)]
pub struct ServiceResponse {
    success: bool,
}
