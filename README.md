# Mimimum Crappy Playable 2

A minimum working example of bevy wasm.

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
* Hit `W`, `A`, `S`, `D` keys to move the ball. (You may first need to click the canvas to focus)
