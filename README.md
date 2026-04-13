# Gameplay Sandbox

A simple top-down shooter built with **Rust** and **Bevy**.

This project is a personal playground to explore game development concepts:
- ECS (Entity Component System)
- basic AI
- input handling
- game states
- UI overlays
- lightweight gameplay juice and feedback

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

## Project Structure

- `main.rs` – app wiring and schedule setup
- `camera.rs` – camera setup and screen shake
- `game_state.rs` – game state and restart message
- `player.rs` – player components, movement, shooting, respawn
- `enemy.rs` – enemy spawning and steering
- `combat.rs` – bullets, collisions, cleanup
- `effects.rs` – death particles, score popups, death sequence timing
- `movement.rs` – shared velocity-based movement
- `ui.rs` – HUD and Game Over overlay

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
