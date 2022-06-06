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
    "operation": "user/list"
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
        await websocket.send(jsom.dumps(list_users))
        print(await websocket.recv())

asyncio.run(hello("wss://management-backend.staging.dvb.solutions"))
