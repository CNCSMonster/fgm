# fgm : fast go version manager

> Note: now it's suggested to use [mise](https://github.com/jdx/mise) instead. Because mise can totally replace fgm and do better, so i decide to give up this project.

this project is inspired by [fnm](https://github.com/Schniz/fnm)

## Installation

`cargo install fgm` or `cargo binstall fgm`(Suggested)

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
