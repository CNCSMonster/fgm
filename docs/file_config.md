Following the XDG Base Directory Specification,
the configuration file is located at `~/.fgm/config.toml` when the environment variable `$XDG_CONFIG_HOME` is not set or `$XDG_CONFIG_HOME/fgm/config.toml`.

The configuration file is a TOML file that contains the following fields:

```toml
# set the directory where the go installation packages will be downloaded
installations_dir="/usr/local/share/fgm"
# set where the go installation package 's link will be stored
gate_path="~/.local/share/fgm/go"
# set the web url where the go installation packages's download link will be fetched
remote_source="https://go.dev/dl/"
```
