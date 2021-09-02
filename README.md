# server-wrapper
A simple wrapper application for Minecraft servers supporting auto-restarts and loading data from remote sources such as GitHub actions.

## Using
Running the wrapper requires two config files, `config.toml` and `destinations.toml`.
`config.toml` defines general settings needed for running the wrapper, and `destinations.toml` declares all file sources and where those files should be sourced from.

An example `config.toml` file may look like:
```toml
# Declares the commands to run to start the server. These commands will be run sequentially.
# When the run task exits, the server will restart itself.
run = ["java -jar -Xmx2G fabric-server-launch.jar"]

[tokens]
# An optional GitHub token that is required only if accessing artifacts from GitHub Actions.
github = "<GitHub token>"

[status]
# An optional Discord webhook url that will be posted to when the server starts (or restarts).
webhook = "<Discord webhook url>"

[triggers]
# Declares a named trigger called `startup` that will run on server startup and is used to load new files into appropriate destinations.
# This is currently the only kind of trigger, but in the future webhooks may be supported as trigger.
startup = { type = "startup" }
```

Note: GitHub tokens used for GitHub actions support must have the `workflow` permission enabled!
You can generate a Personal Access Token [here](https://github.com/settings/tokens).

An example `destinations.toml` may look like:
```toml
# Declares a destination with name `mods` that should be placed into the relative path `mods` and be refreshed at the `startup` trigger.
[mods]
path = "mods"
triggers = ["startup"]

# Declare a file source with name `actions` that should unzip the received file with the given filters.
[mods.sources.actions]
transform = { unzip = ["*.jar", "!*-dev.jar", "!*-sources.jar"] }

# Retrieve a mod from GitHub Actions of the given repository and branch.
plasmid = { github = "NucleoidMC/plasmid", branch = "1.16" }

# Declare a file source with the name `jars` that should apply no transform to the loaded files.
[mods.sources.jars]
# Retrieve a mod from a specific URL
fabric-api = { url = "https://github.com/FabricMC/fabric/releases/download/0.26.3%2B1.16/fabric-api-0.26.3+1.16.jar" }

# Declares a destination with name `datapacks` that should be placed into the relative path `world/datapacks` and be refreshed at the `startup` trigger.
[datapacks]
path = "world/datapacks"
triggers = ["startup"]

# Declare a file source with the name `actions` that should apply no transform to the loaded files.
[datapacks.sources.actions]
# Retrieve the datapack zip from the GitHub Actions artifacts of the given repository.
game-configs = { github = "NucleoidMC/Game-Configs" }
```

The basic structure of the destinations file involves the definition of multiple named definitions, where the name can be arbitrary. 
Each destination declares a target path where all files will be copied into. Importantly, **this directory will be cleared**! Make sure to not add anything important to the directory.

Destinations furthermore can declare multiple named sources, where the names are also arbitrary.
The purpose of separate sources is to provide different transform procedures to files. For example, loading from GitHub Actions may require unzipping the artifacts file and selecting a specific file.

Within each source, many specific sources can be declared. The support types are `url`, `github` and `path`.
