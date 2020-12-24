# Yubaba 2D platformer

A 2D wasm platformer.

You control Chihiro and beat Yubaba.

The status of Yubaba is based on [the ranking](https://qiita.com/torifukukaiou/items/c8361231cdc56e493245).

For example,

* One article in the list corresponds to one Yubaba.
* The life of Yubaba is the number of LGTM + 1.
* The attack of Yubaba is the icon of the article authors.

## Demo

https://yushiomote.github.io/mcp2/

* `W`, `A`, `S`, `D` keys to move the character. (You may first need to click the canvas to focus)
* `J` to attack.

## Setup

Install dependencies.

```
make setup
```

## Build

Build wasm. The outputs are placed in `static/target`.

```
make build
```

## Run

Run a http server that shows the game. Check it on your browser.

```
make run
```

## Note

* To change site prefix set the environment variable `MCP2_PREFIX=<prefix>`.
