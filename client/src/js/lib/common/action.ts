export interface ActionData {
  x: number;
  y: number;
  preferY?: boolean;
  fire: boolean;
  id: number;
  deltaTime: number;
}

export class Action {
  x: number;
  y: number;
  preferY: boolean;
  fire: boolean;
  id: number;
  deltaTime: number;

  constructor() {
    this.x = 0;
    this.y = 0;
    this.preferY = false;
    this.fire = false;
    this.id = 0;
    this.deltaTime = 0;
  }

  toJSON(): ActionData {
    return {
      x: this.x,
      y: this.y,
      preferY: this.preferY,
      fire: this.fire,
      id: this.id,
      deltaTime: this.deltaTime
    };
  }

  fromJSON(data: ActionData) {
    this.x = data.x;
    this.y = data.y;
    this.preferY = !!data.preferY;
    this.fire = data.fire ? true : false;
    this.id = data.id || 0;
    this.deltaTime = data.deltaTime;
  }

  clear() {
    this.x = 0;
    this.y = 0;
    this.preferY = false;
    this.fire = false;
    //this.id = 0;
    this.deltaTime = 0;
  }

  set(x: number, y: number, fire: boolean, preferY: boolean = false) {
    this.x = x;
    this.y = y;
    this.preferY = preferY;
    this.fire = fire;
  }

  equals(obj: ActionData) {
    if (this.x === obj.x && this.y === obj.y && this.fire === obj.fire) {
      return true;
    }

    return false;
  }
}
