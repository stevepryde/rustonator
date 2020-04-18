// FLAGS. NOTE: MUST BE POWERS OF 2!!!
export enum PlayerFlags {
  None = 0,
  WalkThroughBombs = 1 << 0,
  Invincible = 1 << 1
}

export interface PlayerFlagsObj {
  bits: PlayerFlags
}

export function castPlayerFlags(x: number): PlayerFlags {
  let bits = PlayerFlags.None;
  let i = 0;
  let perm: number;
  while (PlayerFlags[perm = 1 << i++]) {
    if (x & perm) {
      bits |= perm;
    }
  }

  return bits;
}
