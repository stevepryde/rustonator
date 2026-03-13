export const TIMED_BOMB_FRAME_COUNT = 4;
export const TIMED_BOMB_FRAME_DURATION_MS = 1000;

export function getTimedBombStartFrame(remaining: number): number {
  const secsRemaining = Math.max(0, Math.min(TIMED_BOMB_FRAME_COUNT, Math.floor(remaining)));
  return Math.min(TIMED_BOMB_FRAME_COUNT - 1, Math.max(0, TIMED_BOMB_FRAME_COUNT - secsRemaining));
}

export function getTimedBombFrame(remaining: number, elapsedMs: number): number {
  const startFrame = getTimedBombStartFrame(remaining);
  const elapsedFrames = Math.max(0, Math.floor(elapsedMs / TIMED_BOMB_FRAME_DURATION_MS));
  return Math.min(TIMED_BOMB_FRAME_COUNT - 1, startFrame + elapsedFrames);
}
