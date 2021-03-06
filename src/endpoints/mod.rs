mod region;
mod station;
mod user;

pub use super::{Region, Role, Station, User, UserConnection};

pub use station::{
    approve_station, create_station, delete_station, generate_token, list_stations, modify_station,
    ApproveStation, CreateStationRequest, ListStationsRequest, ModifyStation,
};
pub use user::{
    create_user, delete_user, get_session, login, modify_user, list_users,
    LoginRequest, ModifyUserRequest,
    RegisterUserRequest, UuidRequest,
};

pub use region::{
    create_region, delete_region, list_regions, modify_region, ModifyRegionRequest, RegionRequest,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct IdentifierRequest {
    pub id: u32,
}

#[derive(Serialize)]
pub struct ServiceResponse {
    success: bool,
    message: Option<String>
}
