# fgm : fast go version manager

this project is inspired by [fnm](https://github.com/Schniz/fnm)

## Quick Start

add the following to your shell profile:

```sh
eval $(fgm init)
```

then you can use `fgm` to install go versions:

```sh
fgm install 1.16.3
fgm use 1.16.3
go version
```