# Node

### Why Diesel?
Diesel is an ORM and completely type safe.
Diesel is [fast](https://github.com/diesel-rs/metrics/).
We only need to do (for the most part) basic CRUD operations.

### Why Postgres?
Fast, reliable and production ready.
Will immediately get a performance boost if using timescaledb.

### TO-DO
- As soon as [OnceCell](https://docs.rs/tokio/latest/tokio/sync/struct.OnceCell.html) is a stable feature, replace lazy_static.
- Once trie is implemented remove delta-tree as default feature.
- Implement tracing for logging.
- Possibly rename entities, since name is a little ambiguous
- If trie structs differ from delta-tree structs, create new subfolders in database/entities (like in handlers)
- Add metrics, Prometheus compatible if possible
- ~~Seperate router setup to routes.rs files~~
- Maybe remove fern as logging setup? Depends on added complexity.
- ~~Move models from database/entities to domain/models~~
- Move models into their own domain/models subfolder depending on where they're used
- Change sync task to subscribe to event stream instead of querying node
- Add match statements in insert functions so for example Transfer and TransferModel can be passed as arguments
- Implement clean shutdown, stopping TcpListeners and closing database connections (catch CTRL+C signal)
- Move diesel.toml into diesel folder, renaming it to be delta-tree specific
- Rename the tables in diesel/delta-tree/migrations so that they're prefixed with delta_
- Add license

### Testing
In order to test, make sure you have [cargo-nextest](https://nexte.st) and [docker-compose](https://docs.docker.com/compose/install/#scenario-two-install-the-compose-plugin) installed.
You might also need the [jq](https://jqlang.github.io/jq/) cli tool. It comes preinstalled on most linux distros.
Executing `cargo nextest run` will automatically spawn a network using CCTL and a postgresql database.
The environment will stay running after test execution ends until explicitly stopped using the command `docker-compose down` or `docker compose down`. The reasoning behind this is to keep the time waiting on the images to spin up to a minimum while developing and testing the code.