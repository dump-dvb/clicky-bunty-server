{pkgs, config, lib, makeTest}: 
  (import "${pkgs}/nixos/make-test-python.nix" {
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
      environment.systemPackages = [
        pkgs.curl 
        (pkgs.writers.writePython3Bin "do_test"
          {
            libraries = [ pkgs.python3Packages.websockets];
            flakeIgnore = [
              # We don't live in the dark ages anymore.
              # Languages like Python that are whitespace heavy will overrun
              # 79 characters..
              "E501"
            ];
          } ''
            import asyncio
            import json
            from websockets import connect

            create_user = {
              "operation": "user/register",
              "body": {
                "name": "test_user",
                "email": "test@test.com",
                "password": "test"
              }
            }

            fetch_session = {
              "operation": "user/session"
            }

            raw_config = json.dumps(config);

            async def hello(uri):
                async with connect(uri) as websocket:
                    await websocket.send(create_user)
                    reigster_response = await websocket.recv()
                    await websocket.send(fetch_session)

            asyncio.run(hello("ws://127.0.0.1:8090"))
          ''
        )
      ];
    };
  };
  testScript = ''
    start_all()
    server.wait_for_unit("clicky-bunty-server.service")
    server.wait_for_open_port(8090)
    do_test()
  '';

  })


