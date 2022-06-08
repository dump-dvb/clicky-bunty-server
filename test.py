#!/usr/bin/env nix-shell
#!nix-shell --pure -i python3.9 -p "python39Packes.ghcWithPackages (pkgs: [ pkgs.turtle ])"

#"{\"operation\": \"user/register\", \"tag\": \"Register\", \"body\": {\"tag\": \"Register\", \"name\":\"test\", \"password\": \"test\", \"email\": \"test@test.com\"}}"
import asyncio
import json
from websockets import connect
import random, string

def randomword(length):
   letters = string.ascii_lowercase
   return ''.join(random.choice(letters) for i in range(length))

create_admin_user = {
    "operation": "user/register",
    "body": {
        "name": "test",
        "password": "test",
        "email": "test@test.com"
    }
}

login_admin = {
    "operation": "user/login",
    "body": {
        "name": "test",
        "password": "test"
    }
}

create_regular_user = {
    "operation": "user/register",
    "body": {
        "name": randomword(12),
        "password": randomword(12),
        "email": "test@test.com"
    }
}

list_users = {
    "operation": "user/list",
}

get_session = {
    "operation": "user/session"
}

create_region = {
    "operation": "region/create",
    "body": {
        "name": "dresden",
        "frequency": 173000000,
        "transport_company": "dresdner verkehrs betriebe",
        "protocol": ""
    }
}

list_regions = {
    "operation": "region/list"
}

modify_region = {
    "operation":"region/modify",
    "body": {

    }
}

create_station = {
    "operation": "station/create",
    "body": {
        "name": "Dresden Station Pieschen",
        "lat": 0.0,
        "lon": 0.0,
        "region": 1
    }
}

list_stations = {
    "operation": "station/list"
}


list_stations = {
    "operation": "station/list",
    "body": {
        "desired_region": 0
    }
}

async def hello(uri):
    async with connect(uri) as websocket:
        print("Request!")
        await websocket.send(json.dumps(create_admin_user))
        print(await websocket.recv())
        await websocket.send(json.dumps(create_regular_user))
        print(await websocket.recv())
        await websocket.send(json.dumps(login_admin))
        print(await websocket.recv())
        await websocket.send(json.dumps(list_users))
        print(await websocket.recv())

        await websocket.send(json.dumps(get_session))
        session = await websocket.recv()
        print("User Id:", session)

        await websocket.send(json.dumps(create_region))
        print(await websocket.recv())
        await websocket.send(json.dumps(list_regions))
        print(await websocket.recv())

        await websocket.send(json.dumps(create_station))
        print(await websocket.recv())
        await websocket.send(json.dumps(list_stations))
        print(await websocket.recv())


asyncio.run(hello("wss://management-backend.staging.dvb.solutions"))
