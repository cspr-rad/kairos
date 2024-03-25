# Node

### Why Diesel?
Diesel is an ORM and completely type safe.
Diesel is [fast](https://github.com/diesel-rs/metrics/).
We only need to do (for the most part) basic CRUD operations.

### Why Postgres?
Fast, reliable and production ready.
Will immediately get a performance boost if using timescaledb.

### TO-DO
Nit-picky:
- Rename domain, it's a little ambiguous.
- Fix imports, messy in most files.

Short term:
- (Layer error handler)['https://docs.rs/axum/latest/axum/error_handling/index.html']
- Thiserror custom error types for handlers/delta_tree
- Get rid of casper_types in kairos-risc0-types
- Once trie is implemented remove delta-tree as default feature.
- Implement tracing for logging.
- Move models/types/request structs into seperate crate so CLI and others can use them too
- Rename the tables in diesel/delta-tree/migrations so that they're prefixed with `delta_`
- Change sync task to also subscribe to event stream on top of querying node
- Move diesel.toml into diesel folder, renaming it to be delta-tree specific
- Add match statements in insert functions so for example Transfer and TransferModel can be passed as arguments
- Add license
- Seperate out database into seperate crate (maybe)

Long term:
- As soon as [OnceCell](https://docs.rs/tokio/latest/tokio/sync/struct.OnceCell.html) is a stable feature, replace lazy_static.

Done:
- ~~Seperate router setup to routes.rs files~~
- ~~Move models from database/entities to domain/models~~
- ~~Implement clean shutdown, stopping TcpListeners and closing database connections (catch CTRL+C signal)~~
- ~~Applying database migrations~~
- ~~Add metrics, Prometheus compatible if possible~~


### Testing
In order to test, make sure you have [cargo-nextest](https://nexte.st) and [docker-compose](https://docs.docker.com/compose/install/#scenario-two-install-the-compose-plugin) installed.
You might also need the [jq](https://jqlang.github.io/jq/) cli tool. It comes preinstalled on most linux distros.
Executing `cargo nextest run` will automatically spawn a network using CCTL and a postgresql database.
The environment will stay running after test execution ends until explicitly stopped using the command `docker-compose down` or `docker compose down`. The reasoning behind this is to keep the time waiting on the images to spin up to a minimum while developing and testing the code.