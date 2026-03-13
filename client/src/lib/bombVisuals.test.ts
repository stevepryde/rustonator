import { describe, expect, test } from "bun:test";
import { getTimedBombFrame, getTimedBombStartFrame } from "./bombVisuals";

describe("getTimedBombStartFrame", () => {
  test("starts fresh bombs on the first frame", () => {
    expect(getTimedBombStartFrame(4.8)).toBe(0);
    expect(getTimedBombStartFrame(4.0)).toBe(0);
  });

  test("maps mid-life bombs to later frames", () => {
    expect(getTimedBombStartFrame(3.9)).toBe(1);
    expect(getTimedBombStartFrame(2.4)).toBe(2);
  });

  test("starts low-time bombs on the last frame", () => {
    expect(getTimedBombStartFrame(1.9)).toBe(3);
    expect(getTimedBombStartFrame(0.2)).toBe(3);
  });
});

describe("getTimedBombFrame", () => {
  test("keeps fresh bombs on the same frame for roughly one second", () => {
    expect(getTimedBombFrame(4.8, 0)).toBe(0);
    expect(getTimedBombFrame(4.8, 999)).toBe(0);
    expect(getTimedBombFrame(4.8, 1000)).toBe(1);
  });

  test("preserves late-start buckets and advances once per second locally", () => {
    expect(getTimedBombFrame(3.9, 0)).toBe(1);
    expect(getTimedBombFrame(3.9, 999)).toBe(1);
    expect(getTimedBombFrame(3.9, 1000)).toBe(2);
    expect(getTimedBombFrame(1.9, 2500)).toBe(3);
  });
});
