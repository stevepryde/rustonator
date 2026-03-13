import { describe, expect, test } from "bun:test";
import { resolveDirectionalAnimationKey } from "./spriteAnimations";

describe("resolveDirectionalAnimationKey", () => {
  test("uses dedicated vertical mob animation for both up and down", () => {
    expect(resolveDirectionalAnimationKey("mob1", "up")).toBe("mob1_updown");
    expect(resolveDirectionalAnimationKey("mob1", "down")).toBe("mob1_updown");
  });

  test("keeps directional animation keys for players and horizontal mob movement", () => {
    expect(resolveDirectionalAnimationKey("p1", "up")).toBe("p1_up");
    expect(resolveDirectionalAnimationKey("p1", "down")).toBe("p1_down");
    expect(resolveDirectionalAnimationKey("mob1", "left")).toBe("mob1_left");
    expect(resolveDirectionalAnimationKey("mob1", "right")).toBe("mob1_right");
  });
});
