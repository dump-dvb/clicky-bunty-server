mod region;
mod station;
mod user;

pub use super::{Region, Role, Station, User, UserConnection};

pub use station::{
    approve_station, create_station, delete_station, generate_token, list_stations, modify_station,
    ApproveStation, CreateStationRequest, ListStationsRequest, ModifyStation,
};
pub use user::{
    create_user, delete_user, get_session, login, modify_user, LoginRequest, ModifyUserRequest,
    RegisterUserRequest, UserIdentifierRequest,
};

pub use region::{
    create_region, delete_region, list_regions, modify_region, ModifyRegionRequest, RegionRequest,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct IdentifierRequest {
    pub id: u32,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Body {
    Empty,
    Register(RegisterUserRequest),
    Login(LoginRequest),
    UserModify(ModifyUserRequest),
    UserIdentifier(UserIdentifierRequest),

    CreateStation(CreateStationRequest),
    ListStation(ListStationsRequest),
    ModifyStation(ModifyStation),
    ApproveStation(ApproveStation),

    ListStations(ListStationsRequest),
    CreateRegion(RegionRequest),
    ModifyRegion(ModifyRegionRequest),

    Identifier(IdentifierRequest),
}

#[derive(Serialize)]
pub struct ServiceResponse {
    success: bool,
}
