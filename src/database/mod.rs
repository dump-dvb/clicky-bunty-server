extern crate postgres;

use postgres::{Client, NoTls, config::SslMode };
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::cmp::PartialEq;
use std::env;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Role {
    User = 6,
    Administrator = 0,
}


impl Role {
    pub fn from(role: u32) -> Role {
        match role {
            0 => Role::Administrator,
            _ => Role::User,
        }
    }

    pub fn as_int(&self) -> u32 {
        match self {
            Role::Administrator => 0,
            _ => 6,
        }
    }
}


#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: Role,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == Role::Administrator
    }
}

#[derive(Serialize, Debug)]
pub struct Region {
    pub id: u32,
    pub name: String,
    pub transport_company: String,
    pub frequency: u64,
    pub protocol: String,
}

pub struct Station {
    pub id: Uuid,
    pub token: Option<String>,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub region: u32,
    pub owner: Uuid,
    pub approved: bool,
}

pub struct DataBaseConnection {
    postgres: Client,
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("User", 4)?;
        s.serialize_field("id", &self.id.to_string())?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("email", &self.email)?;
        s.serialize_field("password", &self.password)?;
        s.end()
    }
}

impl Serialize for Station {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Station", 8).unwrap();
        s.serialize_field("token", &self.token)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("lat", &self.lat)?;
        s.serialize_field("lon", &self.lon)?;
        s.serialize_field("region", &self.region)?;
        s.serialize_field("owner", &self.owner.to_string())?;
        s.serialize_field("approved", &self.approved)?;
        s.end()
    }
}

impl DataBaseConnection {
    pub fn new() -> DataBaseConnection {
        let default_postgres_host = String::from("localhost:5433");
        let default_postgres_port = String::from("5432");

        let postgres_host = format!(
            "posgresql://dvbdump:{}@{}:{}/dvbdump",
            env::var("POSTGRES_PASSWORD").unwrap(),
            env::var("POSTGRES_HOST").unwrap_or(default_postgres_host.clone()),
            env::var("POSTGRES_PORT").unwrap_or(default_postgres_port.clone())
        );

        println!("Connecting to Database at {}", postgres_host);
        let mut database = DataBaseConnection {
            postgres: Client::configure()
                .user("dvbdump")
                .password(env::var("POSTGRES_PASSWORD").unwrap())
                .dbname("dvbdump")
                .host(&env::var("POSTGRES_HOST").unwrap_or(default_postgres_host))
                .port(env::var("POSTGRES_PORT").unwrap_or(default_postgres_port).parse::<u16>().unwrap())
                .ssl_mode(SslMode::Disable)
                .connect(NoTls).unwrap(),
        };
        println!("Creating Database Tables !");
        database.create_tables();

        return database;
    }

    pub  fn create_tables(&mut self) {
        match self.postgres
            .execute(
                "CREATE TABLE users (
                    id              UUID PRIMARY KEY,
                    name            TEXT NOT NULL,
                    email           TEXT NOT NULL,
                    password        VARCHAR(100) NOT NULL,
                    role            INT NOT NULL
                  )",
                &[],
        ) {
            Err(e) => {println!("Did not create table user maybe it already exists! {:?}", e); },
            _ => {}
        }

        match self.postgres
            .execute(
                "CREATE TABLE regions (
                    id              SERIAL PRIMARY KEY,
                    name            TEXT NOT NULL,
                    transport_company TEXT NOT NULL,
                    frequency       BIGINT NOT NULL,
                    protocol        TEXT NOT NULL
                  )",
                &[],
        ) {
            Err(e) => {println!("Did not create table regions maybe it already exists! {:?}", e); },
            _ => {}
        }

        match self.postgres
            .execute(
                "CREATE TABLE stations (
                    id              UUID PRIMARY KEY,
                    token           VARCHAR(32),
                    name            TEXT NOT NULL,
                    lat             DOUBLE PRECISION NOT NULL,
                    lon             DOUBLE PRECISION NOT NULL,
                    region          INT REFERENCES regions(id) NOT NULL,
                    owner           UUID REFERENCES users(id) NOT NULL,
                    approved        BOOLEAN NOT NULL
                  )",
                &[],
        ) {
            Err(e) => {println!("Did not create table stations maybe it already exists! {:?}", e);},
            _ => { }
        }
    }

    pub  fn query_station(&mut self, token: &Uuid) -> Option<Station> {
        match self.postgres.query_one(
            "SELECT token, id, name, lat, lon, region, owner, approved FROM stations WHERE id=$1",
            &[&token],
        ) {
            Ok(data) => Some(Station {
                token: Some(data.get(0)),
                id: Uuid::parse_str(data.get(1)).unwrap(),
                name: data.get(2),
                lat: data.get(3),
                lon: data.get(4),
                region: data.get(5),
                owner: Uuid::parse_str(data.get(6)).unwrap(),
                approved: data.get(7),
            }),
            _ => None,
        }
    }

    pub  fn query_region(&mut self, id: &u32) -> Option<Region> {
        match self.postgres.query_one(
            "SELECT id, name, transport_company, frequency, protocol FROM stations WHERE id=$1",
            &[id],
        ) {
            Ok(data) => Some(Region {
                id: data.get(0),
                name: data.get(1),
                transport_company: data.get(2),
                frequency: data.get::<usize, i64>(3) as u64,
                protocol: data.get(4),
            }),
            _ => None,
        }
    }

    pub  fn query_user(&mut self, name: &String) -> Option<User> {
        match self.postgres.query_one(
            "SELECT id, name, email, password, role FROM users WHERE name=$1",
            &[&name],
        ) {
            Ok(data) => {
                let user_id: Uuid = data.get(0);
                let role: i32 = data.get(4);
                println!("data: {:?}", data);
                Some(User {
                    id: user_id,
                    name: data.get(1),
                    email: data.get(2),
                    password: data.get(3),
                    role: Role::from(role as u32),
                })
            },
            Err(e) => {
                println!("query user {:?}", e);
                None
            }
        }
    }

    pub  fn query_user_by_id(&mut self, id: &Uuid) -> Option<User> {
        match self.postgres.query_one(
            "SELECT id, name, email, password FROM users WHERE id=$1",
            &[id],
        ) {
            Ok(data) => Some(User {
                id: Uuid::parse_str(data.get(0)).unwrap(),
                name: data.get(1),
                email: data.get(2),
                password: data.get(3),
                role: Role::from(data.get(4)),
            }),
            _ => None,
        }
    }
    pub fn check_region_exists(&mut self, id: u32) -> bool {
        match self
            .postgres
            .query("SELECT 1 FROM regions WHERE id=$1", &[&id])
        {
            Ok(data) => {data.len() > 0}
            _ => true,
        }
    }

    pub fn check_user_exists(&mut self, name: &String) -> bool {
        match self
            .postgres
            .query("SELECT 1 FROM users WHERE name=$1", &[name])
        {
            Ok(data) => {
                println!("Users exists: {}", data.len());
                data.len() > 0
            },
            Err(e) => {
                // illegal state has most likely happend prohibit login
                println!("Exists error: {:?}", e);
                true
            }
        }
    }
    pub  fn list_stations(
        &mut self,
        owner: Option<String>,
        region: Option<u32>,
    ) -> Vec<Station> {
        let mut station_list: Vec<Station> = Vec::new();
        let argumnet_count = owner.clone().map_or_else(|| 0, |_| 1) + region.map_or_else(|| 0, |_| 1);

        let owner_query = owner.clone().map_or_else(
            || String::from(""),
            |_| format!("WHERE owner=${} ", argumnet_count - 1),
        );
        let region_query = region.map_or_else(
            || String::from(""),
            |_| format!("WHERE region=${}", argumnet_count),
        );

        let query = format!(
            "SELECT id, name, lat, lon, region, owner, approved FROM stations {}{}",
            owner_query, region_query
        );

        let results;

        println!("Query {}", &query);
        if owner.is_some() && region.is_some() {
            results = self
                .postgres
                .query(&query, &[&(owner.unwrap()), &(region.unwrap() as i32)]);
        } else if owner.is_some() {
            results = self
                .postgres
                .query(&query, &[&(owner.unwrap())]);
        } else if region.is_some() {
            results = self.postgres.query(&query, &[&(region.unwrap() as i32)]);
        } else {
            results = self.postgres.query(&query, &[]);
        }
        match results {
            Ok(data) => {
                for row in data {
                    let station_id: Uuid = row.get(0);
                    let region: i32 = row.get(4);
                    let owner: Uuid = row.get(5);

                    station_list.push(Station {
                        id: station_id,
                        token: None,
                        name: row.get(1),
                        lat: row.get(2),
                        lon: row.get(3),
                        region: region as u32,
                        owner: owner,
                        approved: row.get(6),
                    });
                }
            }
            Err(e) => {
                println!("Error on searching stations {:?}",e);
            }
        }

        station_list
    }

    pub fn list_regions(&mut self) -> Vec<Region> {
        let mut results = Vec::new();
        for row in self
            .postgres
            .query(
                "SELECT id, name, transport_company, frequency, protocol FROM regions",
                &[],
            )
            .unwrap()
        {
            results.push(Region {
                id: row.get::<usize, i32>(0) as u32,
                name: row.get(1),
                transport_company: row.get(2),
                frequency: row.get::<usize, i64>(3) as u64,
                protocol: row.get(4),
            });
        }
        results
    }

    pub fn list_users(&mut self) -> Vec<User> {
        let mut results = Vec::new();
        match self
            .postgres
            .query(
                "SELECT id, name, email, role FROM users",
                &[],
            ) {
            Ok(data) => {
                for row in data {
                    let user_id: Uuid = row.get(0);
                    let role: i32 = row.get(3);
                    results.push(User {
                        id: user_id,
                        name: row.get(1),
                        email: row.get(2),
                        password: String::from(""),
                        role: Role::from(role as u32),
                    });
                }
            }
            _ => {}

        }
        results
    }


    pub  fn create_user(&mut self, user: &User) -> bool {
        println!("create user: {:?}", &user);
        match self.postgres
            .execute(
                "INSERT INTO users (id, name, email, password, role) VALUES ($1, $2, $3, $4, $5)",
                &[
                    &user.id,
                    &user.name,
                    &user.email,
                    &user.password,
                    &(user.role.as_int() as i32),
                ],
            ) {
                Ok(_) => { true }
                Err(e) => {
                    println!("Error: {}", e);
                    false
                }
        }
    }

    pub fn create_region(&mut self, region: &Region) -> bool {
        println!("region: {:?}", &region);
        match self.postgres
            .execute(
                "INSERT INTO regions (name, transport_company, frequency, protocol) VALUES ($1, $2, $3, $4)",
                &[
                    &region.name,
                    &region.transport_company,
                    &(region.frequency as i64),
                    &region.protocol,
                ],
            ) {
            Ok(respone) => {
                println!("Response: create region: {:?}", respone);
                true 
            }
            Err(e) => {
                println!("Error: {}", e);
                false
            }
        }
    }

    pub  fn create_station(&mut self, station: &Station) -> bool {
        match self.postgres.execute(
            "INSERT INTO stations (id, token, name, lat, lon, region, owner, approved) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &[
                &station.id,
                &station.token,
                &station.name,
                &station.lat,
                &station.lon,
                &(station.region as i32),
                &station.owner,
                &station.approved
            ],
        ) {
            Ok(_) => { true }
            Err(e) => {
                println!("Error: {}", e);
                false
            }
        }
    }

    pub  fn first_user(&mut self) -> bool {
        match self.postgres.query("SELECT 1 FROM users", &[]) {
            Ok(data) => { data.len() == 0 },
            Err(_) => false,
        }
    }

    pub  fn is_administrator(&mut self, uid: &Uuid) -> bool {
        match self
            .postgres
            .query_one("SELECT role FROM users WHERE id=$1", &[uid])
        {
            Ok(row) => row.get::<usize, i32>(0) == 0,
            _ => false,
        }
    }

    pub  fn get_owner_from_station(&mut self, station_id: &Uuid) -> Option<Uuid> {
        match self.query_station(station_id){
            Some(region) => Some(region.owner),
            _ => None,
        }
    }

    pub  fn delete_user(&mut self, uid: &Uuid) -> bool {
        self.postgres
            .execute("DELETE FROM users WHERE id=$1", &[uid])
            .is_ok()
    }

    pub  fn delete_region(&mut self, id: &u32) -> bool {
        self.postgres
            .execute("DELETE FROM users WHERE id=$1", &[id])
            .is_ok()
    }

    pub  fn delete_station(&mut self, id: &Uuid) -> bool {
        self.postgres
            .execute("DELETE FROM users WHERE id=$1", &[id])
            .is_ok()
    }

    pub  fn update_user(&mut self, user: &User) -> bool {
        self.postgres
            .execute(
                "UPDATE users SET name=$1, email=$2, password=$3, role=$4 WHERE id=$5",
                &[
                    &user.name,
                    &user.email,
                    &user.password,
                    &user.role.as_int(),
                    &user.id.to_string(),
                ],
            )
            .is_ok()
    }

    pub  fn update_station(&mut self, station: &Station) -> bool {
        self.postgres
            .execute(
                "UPDATE station SET name=$1, lat=$2, lon=$3, region=$4 WHERE id=$5",
                &[
                    &station.name,
                    &station.lat,
                    &station.lon,
                    &station.region,
                    &station.id,
                ],
            )
            .is_ok()
    }

    pub  fn update_region(&mut self, region: &Region) -> bool {
        self.postgres.execute("UPDATE region SET name=$1, transport_company=$2, frequency=$3, protocol=$4 WHERE id=$5",
                              &[&region.name, &region.transport_company, &(region.frequency as i64), &region.protocol, &(region.id as i64)]).is_ok()
    }

    pub fn set_approved(&mut self, id: &Uuid, approved: bool) -> bool {
        match self.postgres
            .execute(
                "UPDATE station SET approved=$1 WHERE id=$2",
                &[&approved, id],
            ) {
            Ok(data) => {
                println!("Approve: {:?}", data);
                true
            }
            Err(e) => {
                println!("Approve: {:?}", e);
                false
            }
        }
    }

    pub fn set_token(&mut self, id: &Uuid, token: &String) -> bool {
        self.postgres
            .execute("UPDATE station SET token=$1 WHERE id=$2", &[token, id])
            .is_ok()
    }
}
