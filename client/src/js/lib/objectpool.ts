export class ObjectPool<T> {
  items: { [x: string]: T };
  valid: { [x: string]: boolean };

  constructor() {
    this.items = {};
    this.valid = {};
  }

  clear() {
    this.items = {};
    this.valid = {};
  }

  mark() {
    this.valid = {};
  }

  set(id: string, item: T) {
    this.items[id] = item;
    this.valid[id] = true;
  }

  get(id: string): T | null {
    return this.items.hasOwnProperty(id) ? this.items[id] : null;
  }

  cleanUp(fn: (id: string) => void) {
    let keep: { [x: string]: T } = {};
    Object.keys(this.items).forEach((id: string) => {
      if (!this.valid.hasOwnProperty(id)) {
        fn(id);
      } else {
        keep[id] = this.items[id];
      }
    });

    this.items = keep;
  }
}
