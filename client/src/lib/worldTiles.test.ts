import { describe, expect, test } from "bun:test";
import CellType from "../js/lib/common/celltypes";
import { getWorldTileFrame } from "./worldTiles";

describe("getWorldTileFrame", () => {
  test("renders bomb cells as empty terrain because bombs have their own sprite layer", () => {
    expect(getWorldTileFrame(CellType.Bomb)).toBe(CellType.Empty);
  });

  test("keeps normal terrain and item frames unchanged", () => {
    expect(getWorldTileFrame(CellType.Wall)).toBe(CellType.Wall);
    expect(getWorldTileFrame(CellType.Mystery)).toBe(CellType.Mystery);
    expect(getWorldTileFrame(CellType.ItemBomb)).toBe(CellType.ItemBomb);
  });
});
