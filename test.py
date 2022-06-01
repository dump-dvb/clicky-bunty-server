#!/usr/bin/env nix-shell
#!nix-shell --pure -i python3.9 -p "python39Packes.ghcWithPackages (pkgs: [ pkgs.turtle ])"

import asyncio
import json
from websockets import connect

create_user = {
    "operation": "user/register",
    "body": {
        "name": "test",
        "password": "test",
        "email": "test@test.com"
    }
}

raw_config = json.dumps(create_user);

async def hello(uri):
    async with connect(uri, ssl=None) as websocket:
        print("Request!")
        await websocket.send(raw_config)
        print(await websocket.recv())

asyncio.run(hello("wss://management-backend.staging.dvb.solutions"))
