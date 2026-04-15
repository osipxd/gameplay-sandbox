# Gameplay Sandbox

A simple top-down shooter built with **Rust** and **Bevy**.

This project is a personal playground to explore game development concepts:
- ECS (Entity Component System)
- basic AI
- input handling
- game states
- UI overlays
- lightweight gameplay juice and feedback
- procedural textures and variable-font text styling

## Features

- Player movement (WASD)
- Directional shooting (arrow keys)
- Enemies that follow the player
- Enemy spawning over time
- Collision system (bullets vs enemies, enemies vs player)
- Player health system
- Short invincibility window after taking damage
- Screen shake on player hit
- Death particles for enemies and the player
- Score popups on enemy kills
- Game Over screen with restart button
- HUD with HP and score
- Score tracking and reset on restart
- Procedural face textures for player and enemies
- Screen-space vignette overlay
- Variable fonts: Inter for UI, JetBrains Mono for score popups

## Controls

- Move: `W`, `A`, `S`, `D`
- Shoot: Arrow keys

## Tech Stack

- [Rust](https://www.rust-lang.org/)
- [Bevy Engine](https://bevyengine.org/)

## Running the project

Make sure you have Rust installed.

```bash
cargo run
```

For local debugging with Bevy's extra debug feature enabled:

```bash
cargo dev-run
```

This dev command also enables Bevy's asset file watcher, so supported asset changes
hot-reload while the desktop game is running. Right now that includes `assets/effects.ron`.

## Running in the browser

Install the required wasm tooling once:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

Build the web bundle:

```bash
./scripts/build-web.sh
```

Serve it locally:

```bash
./scripts/serve-web.sh
```

Then open [http://localhost:8000](http://localhost:8000).

## Docker deployment

Build the production image:

```bash
docker build -t gameplay-sandbox:latest .
```

Run it with Caddy serving the generated static site:

```bash
docker run --rm -p 8080:80 gameplay-sandbox:latest
```

Then open [http://localhost:8080](http://localhost:8080).

The container uses a multi-stage build:
- Rust builder stage compiles the wasm bundle with `./scripts/build-web.sh release`
- final `caddy:alpine` stage serves `web/` as static files

Deployment files:
- `Dockerfile` – multi-stage web build + minimal runtime image
- `Caddyfile` – static site config with compression and light caching
- `.dockerignore` – trims the Docker build context

## GitHub Container Registry

Publishing to GHCR is handled by [`.github/workflows/publish-image.yml`](.github/workflows/publish-image.yml).

It runs when you push a git tag matching `v*`, logs in with the repository `GITHUB_TOKEN`, and publishes:
- `ghcr.io/osipxd/gameplay-sandbox:<version>`
- `ghcr.io/osipxd/gameplay-sandbox:latest`

Example:

```bash
git tag v0.1.0
git push origin v0.1.0
```

For a real server with a domain, Caddy can obtain and renew Let's Encrypt certificates automatically.
Run the same image with the site address set to your domain, publish ports `80` and `443`, and persist
Caddy state so certificates survive restarts:

```bash
docker run -d \
  --name gameplay-sandbox \
  -e SITE_ADDRESS=game.example.com \
  -p 80:80 \
  -p 443:443 \
  -v caddy_data:/data \
  -v caddy_config:/config \
  ghcr.io/osipxd/gameplay-sandbox:latest
```

When `SITE_ADDRESS` is a real public domain name, Caddy will switch from plain HTTP to automatic HTTPS.
Requirements:
- the domain must resolve to your server
- ports `80` and `443` must be reachable from the internet
- keep `/data` and `/config` persisted for certificate reuse

## Project Structure

- `main.rs` – app wiring and schedule setup
- `camera.rs` – camera setup and screen shake
- `game_state.rs` – game state and restart message
- `player.rs` – player components, movement, shooting, respawn
- `enemy.rs` – enemy spawning and steering
- `combat.rs` – bullets, collisions, cleanup
- `effects.rs` – death particles, score popups, death sequence timing
- `assets/effects.ron` – hot-reloadable effect tuning for particles and score popups
- `movement.rs` – shared velocity-based movement
- `textures.rs` – generated textures for entity faces and the screen vignette
- `ui.rs` – HUD, Game Over overlay, and shared UI font resource
- `.github/workflows/publish-image.yml` – tag-driven GHCR publish workflow
- `web/index.html` – landing page, controls, and project overview
- `web/play.html` – in-browser game page
- `web/style.css` – shared site styles
- `web/game/` – generated wasm bundle output
- `scripts/build-web.sh` – wasm build and asset copy script
- `scripts/serve-web.sh` – local static server for the web build

## Goals

This project is intentionally simple.  
The focus is on learning and experimenting, not building a full game.

## Possible next steps

- Add sound effects
- Introduce difficulty scaling
- Add more enemy types or attack patterns
- Replace simple sprite effects with authored particles or animation data

## License

MIT
