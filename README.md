# Gameplay Sandbox

A simple top-down shooter built with **Rust** and **Bevy**.

This project is a personal playground to explore game development concepts:
- ECS (Entity Component System)
- basic AI
- input handling
- game states
- UI overlays

## Features

- Player movement (WASD)
- Directional shooting (arrow keys)
- Enemies that follow the player
- Enemy spawning over time
- Collision system (bullets vs enemies, enemies vs player)
- Player health system
- Game Over screen with restart button
- Basic UI (HP display)

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


## Project Structure

- `main.rs` – app wiring and schedule setup
- `game_state.rs` – game state and restart message
- `player.rs` – player components, movement, shooting, respawn
- `enemy.rs` – enemy spawning and steering
- `combat.rs` – bullets, collisions, cleanup
- `movement.rs` – shared velocity-based movement
- `ui.rs` – HUD and Game Over overlay

## Goals

This project is intentionally simple.  
The focus is on learning and experimenting, not building a full game.

## Possible next steps

- Add score system
- Improve enemy behavior
- Add sound effects
- Add visual feedback (hit effects, animations)
- Introduce difficulty scaling

## License

MIT
