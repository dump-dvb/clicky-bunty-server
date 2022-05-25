{pkgs, config, lib}: 
{
  "integration-test" = (makeTest {
    name = "integration-test";

    nodes = {
      server = {config, pkgs, ... }:{
        virtualisation.memorySize = 2048;
        services = {
          postgres = {
            enable = true;
            ensureUsers = [
              {
                name = "dvbdump";
                ensurePermissions = {
                  "DATABASE dvbdump" = "ALL PRIVILEGES";
                };
                ensureDatabases = [
                  "dvbdump"
                ];
              }
            ];
          };
        };

      systemd = {
        services = {
          "clicky-bunty-server" = {
            enable = true;
            requires = [ "postgres.service" ];
            after =  [ "postgres.service" ];
            wantedBy = [ "multi-user.target" ];

            script = ''
              exec ${pkgs.clicky-bunty-server}/bin/clicky-bunty-server --host 0.0.0.0 --port 8090
            '';

            environment = {
              "POSTGRES" = "http://localhost:5433";
            };
            serviceConfig = {
              Type = "forking";
              User = "clicky-bunty-server";
              Restart = "always";
            };
          };
        };
      };
  
      users.users = {
        clicky-bunty-server = {
          name = "clicky-bunty-server";
          description = "";
          isNormalUser = true;
        };
      };
    };

    client = {pkgs, config, ...}: {
      environment.systemPackages = [ pkgs.curl ];
    }
  };
  testScript = ''
    start_all()
    server.wait_for_unit("clicky-bunty-server.service")
    server.wait_for_open_port(8090)
    server.succeed("curl --fail http://localhost:8090/")

    #!/usr/bin/env nix-shell
    #!nix-shell --pure -i python3.9 -p "python39Packes.ghcWithPackages (pkgs: [ pkgs.turtle ])"

    import asyncio
    import json
    from websockets import connect

    register = {
      "name": "test",
      "email": "test@test.com",
      "password": "test"
    }

    raw_config = json.dumps(config);

    async def hello(uri):
        async with connect(uri) as websocket:
            await websocket.send(raw_config)
            while True:
                print(await websocket.recv())

    asyncio.run(hello("ws://0.0.0.0:8090"))

  '';

  })



