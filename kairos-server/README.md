# Kairos Server

## Configuration

You can configure this application through environment variables, dotenv or a toml file. Below you can see examples of the latter two. 

**.env**
```
KAIROS_SERVER_PORT="8000"
KAIROS_SERVER_ADDRESS="0.0.0.0"
KAIROS_LOG_LEVEL="trace" # trace, debug, info, warn, error
KAIROS_LOG_FILE="kairos.log" # Optional
```

**kairos-config.toml**
You can also supply the config's path as a command line argument.
```
[server]
address = "0.0.0.0"
port = 8000

[log]
level = "trace" # trace, debug, info, warn, error
file = "kairos.log" # optional
```