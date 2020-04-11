import { Player } from "./player";

export interface BombData {
  id: number;
  pid: string;
  pname: string;
  active: boolean;
  x: number;
  y: number;
  remaining: number;
  range: number;
}

export class Bomb {
  id: number;
  pid: string;
  pname: string;
  active: boolean;
  x: number;
  y: number;
  remaining: number;
  range: number;

  constructor(id: number, player: Player) {
    this.id = id;
    this.pid = player.id;
    this.pname = player.name;
    this.active = true;
    this.x = player.x;
    this.y = player.y;
    this.remaining = player.bombTime;
    this.range = player.range;
  }

  toJSON(): BombData {
    return {
      id: this.id,
      pid: this.pid,
      pname: this.pname,
      active: this.active,
      x: this.x,
      y: this.y,
      remaining: this.remaining,
      range: this.range
    };
  }

  fromJSON(data: BombData): void {
    this.id = data.id;
    this.pid = data.pid;
    this.pname = data.pname;
    this.active = data.active;
    this.x = data.x;
    this.y = data.y;
    this.remaining = data.remaining;
    this.range = data.range;
  }
}
