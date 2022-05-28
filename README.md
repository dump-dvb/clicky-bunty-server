# clicky bunty server

Server which handels users, regions and stations. This service is the main point of operation.


## Building

```bash
    $ nix build
```

## Configuration

- `SALT_PATH` path to file containing the salt that is used for hashing the password
- `POSTGRES` resource identifier for the postgresql

## Documentation 

Can be found [here](https://github.com/dump-dvb/documentation/blob/master/src/chapter_user_api.md).
