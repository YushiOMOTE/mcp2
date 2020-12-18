# Mimimum Crappy Playable 2

A minimum working example of bevy wasm 2d platformer.

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

## Demo

http://chintama.club/mcp2/

## Note

* To change site prefix set the environment variable `MCP2_PREFIX=<prefix>`.
* `W`, `A`, `S`, `D` keys to move the character. (You may first need to click the canvas to focus)
* `E` to switch to `debug mode`.
* `P` to switch to `play mode`.
