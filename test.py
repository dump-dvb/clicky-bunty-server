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
        "body":{
            "id":1,
            "name":"cologne",
            "transport_company":"cologe verkehrs betriebe",
            "frequency":143000002,
            "protocol":"Test Cring"
        }
}

modify_station = {
        "operation":"station/modify",
        "body":{
            "id":"2b4bfa9c-3768-43f4-b472-3a4828e251d6",
            "token":"",
            "name":"Dresden Cringe",
            "lat":0,
            "lon":2,
            "region":1,
            "owner":"7cd2c08b-07c9-453a-a221-80f990fa68a0",
            "approved":True
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
        #"desired_region": 1
    }
}

approve_station = {
    "operation": "station/approve",
    "body": {
        "id": "",
        "approved": True 
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
        print("login_admin", await websocket.recv())
        await websocket.send(json.dumps(list_users))
        print("list users:", await websocket.recv())

        await websocket.send(json.dumps(get_session))
        session = await websocket.recv()
        print("User Id:", session)

        await websocket.send(json.dumps(create_region))
        print("create_region:", await websocket.recv())
        await websocket.send(json.dumps(list_regions))
        print("list_regions:", await websocket.recv())

        await websocket.send(json.dumps(create_station))
        print("Create Statio:", await websocket.recv())
        await websocket.send(json.dumps(list_stations))
        station_list = json.loads(await websocket.recv())
        print("List Stations:", station_list[0])
        approve_station["body"]["id"] = station_list[0]["id"]
        await websocket.send(json.dumps(approve_station))
        print("Approve Station: ",await websocket.recv())

        await websocket.send(json.dumps(modify_region))
        print("Modify Region:", await websocket.recv())

        await websocket.send(json.dumps(modify_station))
        print("Modify Station:", await websocket.recv())


asyncio.run(hello("wss://management-backend.staging.dvb.solutions"))
