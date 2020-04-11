import { Action, ActionData } from "./action";
import CellType from "./celltypes";
import { IAgent } from "./agent";

export interface MobData {
  id: number;
  active: boolean;
  x: number;
  y: number;
  action: ActionData;
  speed: number;
  image: string;
  name: string;
}

export class Mob implements IAgent {
  id: number;
  active: boolean;
  x: number;
  y: number;
  action: Action;
  speed: number;
  image: string;
  name: string;

  constructor() {
    this.id = 0;
    this.active = true;
    this.x = 0;
    this.y = 0;
    this.action = new Action();
    this.speed = 60; // speed (in pixels per second).
    this.image = "mob1"; //Image label.
    this.name = "";
  }

  toJSON(): MobData {
    return {
      id: this.id,
      active: this.active,
      x: this.x,
      y: this.y,
      action: this.action.toJSON(),
      speed: this.speed,
      image: this.image,
      name: this.name
    };
  }

  fromJSON(data: MobData): void {
    this.id = data.id;
    this.active = data.active;
    this.x = data.x;
    this.y = data.y;
    this.action.fromJSON(data.action);
    this.speed = data.speed;
    this.image = data.image;
    this.name = data.name;
  }

  canPass(cellType: CellType): boolean {
    switch (cellType) {
      case CellType.Wall:
      case CellType.Mystery:
      case CellType.Bomb:
        // NOTE: mobs cannot pass through bombs!
        return false;
      default:
        return true;
    }
  }

  updateWithTempAction(tmpaction: ActionData): void {
    if (tmpaction) {
      var effectiveSpeed = this.speed;
      if (effectiveSpeed < 50) {
        effectiveSpeed = 50;
      } else if (effectiveSpeed > 300) {
        effectiveSpeed = 300;
      }
      this.x += tmpaction.x * tmpaction.deltaTime * effectiveSpeed;
      this.y += tmpaction.y * tmpaction.deltaTime * effectiveSpeed;
    }
  }
}
