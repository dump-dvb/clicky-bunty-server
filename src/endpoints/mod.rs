mod region;
mod station;
mod user;

pub use super::{Region, Role, Station, User, UserConnection};

pub use station::{
    approve_station, create_station, delete_station, generate_token, list_stations, modify_station,
    ApproveStation, CreateStationRequest, DeleteStation, GenerateToken, ListStationsRequest,
    ModifyStation,
};
pub use user::{
    create_user, delete_user, get_session, login, modify_user, DeleteUserRequest, LoginRequest,
    ModifyUserRequest, RegisterUserRequest,
};

pub use region::{
    create_region, delete_region, list_regions, modify_region, DeleteRegion, ModifyRegionRequest,
    RegionRequest,
};

use serde::{Deserialize, Serialize};

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
    CreateRegion(RegionRequest),
    DeleteRegion(DeleteRegion),
    ModifyRegion(ModifyRegionRequest),
}

#[derive(Serialize)]
pub struct ServiceResponse {
    success: bool,
}
