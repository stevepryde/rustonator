# Server Test Harness Plan

## Goal

Build a deterministic scenario harness around `RustonatorGame` so gameplay bugs can be locked in as scripted regression tests instead of only low-level unit tests.

## Phase 1

- Add a fixed-tick `ScenarioHarness` for server tests.
- Support adding players, joining them, queueing actions, disconnecting them, and stepping the game loop.
- Expose compact snapshots for players, bombs, explosions, and selected cells.
- Add core invariants:
  - active bombs must have matching world/internal bomb cells
  - bomb cells must point to live bombs
  - `player.cur_bombs` must match the number of active bombs owned by that player

## Phase 2

- Add scenario tests for:
  - timed bomb placement and cleanup
  - remote bomb reversion on death, disconnect, and input drop
  - reverted remote bomb returning to the normal timed explosion lifecycle
  - multi-bomb chain reactions
  - score attribution for self-kills vs enemy kills

## Phase 3

- Add deterministic map fixtures or helpers for special layouts.
- Add optional scripted assertions per tick for hard-to-reproduce regressions.
- Add a nightly soak suite with longer scripted simulations and fixed seeds.

## Guardrails

- Prefer scenario tests over lots of tiny private-method tests when behaviour spans several systems.
- Keep snapshots small and purpose-built so tests stay readable.
- Turn every reproduced gameplay bug into a scenario test before or with the fix.
