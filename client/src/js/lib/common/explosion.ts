import { Bomb } from "./bomb";

export interface ExplosionData {
  id: number;
  pid: string | null;
  pname: string | null;
  active: boolean;
  x: number;
  y: number;
  remaining: number;
  harmful: boolean;
}

export class Explosion {
  id: number;
  pid: string | null;
  pname: string | null;
  active: boolean;
  x: number;
  y: number;
  remaining: number;
  harmful: boolean;

  constructor(id: number, bomb: Bomb | null, ex: number, ey: number) {
    this.id = id;
    this.pid = null;
    this.pname = null;
    if (bomb) {
      this.pid = bomb.pid;
      this.pname = bomb.pname;
    }

    this.active = true;
    this.x = ex;
    this.y = ey;
    this.remaining = 0.5;
    this.harmful = true; // Allow explosions to only be harmful at the start.
  }

  toJSON(): ExplosionData {
    return {
      id: this.id,
      pid: this.pid,
      pname: this.pname,
      active: this.active,
      x: this.x,
      y: this.y,
      remaining: this.remaining,
      harmful: this.harmful
    };
  }

  fromJSON(data: ExplosionData): void {
    this.id = data.id;
    this.pid = data.pid;
    this.pname = data.pname;
    this.active = data.active;
    this.x = data.x;
    this.y = data.y;
    this.remaining = data.remaining;
    this.harmful = data.harmful;
  }
}
