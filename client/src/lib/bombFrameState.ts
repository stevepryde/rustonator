import type { BombData } from "../js/lib/common/bomb";

export interface BombFrameState {
  renderableBombs: BombData[];
  staleIds: string[];
}

export function isRenderableBomb(bomb: BombData | null | undefined): bomb is BombData {
  return !!bomb && bomb.active && bomb.remaining > 0;
}

export function buildBombFrameState(
  bombs: Array<BombData | null | undefined>,
  existingIds: Iterable<string>
): BombFrameState {
  const renderableBombs = bombs.filter(isRenderableBomb);
  const visibleIds = new Set(renderableBombs.map((bomb) => bomb.id.toString()));
  const staleIds: string[] = [];

  for (const id of existingIds) {
    if (!visibleIds.has(id)) {
      staleIds.push(id);
    }
  }

  return {
    renderableBombs,
    staleIds
  };
}

export function reconcileBombStore(
  bombs: Array<BombData | null | undefined>,
  existingIds: Iterable<string>,
  destroyBomb: (id: string) => void
): BombFrameState {
  const state = buildBombFrameState(bombs, existingIds);
  for (const id of state.staleIds) {
    destroyBomb(id);
  }
  return state;
}
