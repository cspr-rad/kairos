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