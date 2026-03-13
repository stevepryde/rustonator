import { describe, expect, test } from "bun:test";
import type { BombData } from "../js/lib/common/bomb";
import { buildBombFrameState, reconcileBombStore } from "./bombFrameState";

function makeBomb(overrides: Partial<BombData> = {}): BombData {
  return {
    id: 1,
    pid: "p1",
    pname: "alice",
    active: true,
    remote: false,
    x: 1,
    y: 1,
    remaining: 3,
    range: 1,
    ...overrides
  };
}

describe("buildBombFrameState", () => {
  test("keeps only active bombs with positive remaining time", () => {
    const state = buildBombFrameState(
      [
        makeBomb({ id: 1, active: true, remaining: 2 }),
        makeBomb({ id: 2, active: false, remaining: 2 }),
        makeBomb({ id: 3, active: true, remaining: 0 })
      ],
      []
    );

    expect(state.renderableBombs.map((bomb) => bomb.id)).toEqual([1]);
  });

  test("marks missing, inactive, and expired bombs as stale", () => {
    const state = buildBombFrameState(
      [
        makeBomb({ id: 1, active: true, remaining: 1.5 }),
        makeBomb({ id: 2, active: false, remaining: 1.5 }),
        makeBomb({ id: 3, active: true, remaining: 0 })
      ],
      ["1", "2", "3", "4"]
    );

    expect(state.staleIds).toEqual(["2", "3", "4"]);
  });
});

describe("reconcileBombStore", () => {
  test("destroys stale client bomb ids while keeping current ones", () => {
    const destroyed: string[] = [];
    const existing = new Set(["1", "2", "9"]);

    const state = reconcileBombStore(
      [
        makeBomb({ id: 1, active: true, remaining: 2 }),
        makeBomb({ id: 2, active: true, remaining: 0 })
      ],
      existing,
      (id) => {
        destroyed.push(id);
        existing.delete(id);
      }
    );

    expect(state.renderableBombs.map((bomb) => bomb.id)).toEqual([1]);
    expect(destroyed).toEqual(["2", "9"]);
    expect(Array.from(existing)).toEqual(["1"]);
  });
});
