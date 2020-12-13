export interface ActionData {
  x: number;
  y: number;
  fire: boolean;
  id: number;
  deltaTime: number;
}

export class Action {
  x: number;
  y: number;
  fire: boolean;
  id: number;
  deltaTime: number;

  constructor() {
    this.x = 0;
    this.y = 0;
    this.fire = false;
    this.id = 0;
    this.deltaTime = 0;
  }

  toJSON(): ActionData {
    return {
      x: this.x,
      y: this.y,
      fire: this.fire,
      id: this.id,
      deltaTime: this.deltaTime
    };
  }

  fromJSON(data: ActionData) {
    this.x = data.x;
    this.y = data.y;
    this.fire = data.fire;
    this.id = data.id || 0;
    this.deltaTime = data.deltaTime;
  }

  clear() {
    this.x = 0;
    this.y = 0;
    this.fire = false;
    //this.id = 0;
    this.deltaTime = 0;
  }

  set(x: number, y: number, fire: boolean) {
    this.x = x;
    this.y = y;
    this.fire = fire;
  }

  equals(obj: ActionData) {
    return this.x === obj.x && this.y === obj.y && this.fire === obj.fire;
  }

  isEmpty(): boolean {
    return this.x === 0 && this.y === 0 && !this.fire;
  }
}
