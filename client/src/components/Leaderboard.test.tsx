import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import Leaderboard from "./Leaderboard";
import type { PlayerData } from "../js/lib/common/player";

function makePlayer(overrides: Partial<PlayerData>): PlayerData {
  return {
    id: "player-1",
    active: true,
    x: 0,
    y: 0,
    action: { id: 0, x: 0, y: 0, fire: false, preferY: false, deltaTime: 0 },
    speed: 200,
    image: "p1",
    range: 1,
    bombTime: 3,
    maxBombs: 1,
    curBombs: 0,
    remoteBombCharges: 0,
    flags: [],
    score: 0,
    scoreMultiplier: 1,
    scoreMultiplierRemaining: 0,
    name: "Player",
    rank: 1,
    effects: [],
    ...overrides,
  };
}

describe("Leaderboard", () => {
  test("renders server-curated leaderboard players even if active is false", () => {
    const markup = renderToStaticMarkup(
      <Leaderboard
        currentPlayerId="player-2"
        players={[
          makePlayer({ id: "player-1", name: "Alice", score: 1200, active: false }),
          makePlayer({ id: "player-2", name: "Bob", score: 800, active: false }),
        ]}
      />
    );

    expect(markup).toContain("Alice");
    expect(markup).toContain("Bob");
    expect(markup).toContain("1200");
    expect(markup).toContain("800");
  });
});
