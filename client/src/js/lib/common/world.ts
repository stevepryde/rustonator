import GameConfig from "./config";
import CellType from "./celltypes";

export interface PositionData {
  x: number;
  y: number;
}

export interface WorldData {
  x: number;
  y: number;
  width: number;
  height: number;
  tilewidth: number;
  tileheight: number;
  chunkwidth: number;
  chunkheight: number;
  data: CellType[];
}

export interface ChunkData {
  tx: number;
  ty: number;
  width: number;
  height: number;
  data: CellType[];
}

export class World {
  x: number;
  y: number;
  width: number;
  height: number;
  tilewidth: number;
  tileheight: number;
  chunkwidth: number;
  chunkheight: number;
  data: CellType[];

  constructor(width?: number, height?: number, config?: GameConfig) {
    this.x = 0;
    this.y = 0;
    this.width = width || 0;
    this.height = height || 0;
    this.tilewidth = 32;
    this.tileheight = 32;
    if (config != null) {
      // Allow extra tiles either side because client-side prediction can
      // have the client several tiles ahead of the server.
      this.chunkwidth = Math.floor(config.screenX / this.tilewidth) + 10;
      this.chunkheight = Math.floor(config.screenY / this.tileheight) + 10;
    } else {
      this.chunkwidth = 32;
      this.chunkheight = 32;
    }

    this.data = [];
  }

  toJSON(): WorldData {
    return {
      x: this.x,
      y: this.y,
      width: this.width,
      height: this.height,
      tilewidth: this.tilewidth,
      tileheight: this.tileheight,
      chunkwidth: this.chunkwidth,
      chunkheight: this.chunkheight,
      data: this.data
    };
  }

  fromJSON(data: WorldData): void {
    this.x = data.x || 0;
    this.y = data.y || 0;
    this.width = data.width;
    this.height = data.height;
    this.tilewidth = data.tilewidth || 32;
    this.tileheight = data.tileheight || 32;
    this.chunkwidth = data.chunkwidth || 35;
    this.chunkheight = data.chunkheight || 28;
    this.data = data.data;
  }

  toCSV(): string {
    var csv = "";

    var index = 0;
    for (var my = 0; my < this.height; my++) {
      csv += this.data.slice(index, index + this.width).join();
      csv += "\n";

      index += this.width;
    }

    return csv;
  }

  isValidCell(x: number, y: number): boolean {
    if (x < 0 || x >= this.width || y < 0 || y >= this.height) {
      return false;
    }

    return true;
  }

  getIndex(x: number, y: number): number {
    if (!this.isValidCell(x, y)) {
      return -1;
    }

    return y * this.width + x;
  }

  setcell(x: number, y: number, val: CellType): void {
    if (!this.isValidCell(x, y)) {
      return;
    }

    this.data[y * this.width + x] = val;
  }

  getcell(x: number, y: number): CellType {
    if (!this.isValidCell(x, y)) {
      return CellType.Wall; // Default to wall.
    }

    return this.data[y * this.width + x];
  }

  getFirstBlank(mx: number, my: number, length: number): number {
    var start = 0;
    if (mx < 1) {
      start = 1 - mx;
      if (start >= length) {
        return -1;
      }
    }

    if (my < 1 || my >= this.height - 1) {
      return -1;
    }

    var index = this.getIndex(mx + start, my);
    for (var i = start; i < length; i++) {
      if (this.data[index++] === CellType.Empty) {
        return mx + i;
      }
    }

    return -1;
  }

  findNearestBlank(mx: number, my: number): PositionData {
    if (this.getcell(mx, my) === 0) {
      return { x: mx, y: my };
    }

    var fallback = { x: 1, y: 1 };

    for (var radius = 1; radius < 20; radius++) {
      var cx = mx - radius;
      var cy = my - radius;

      if (mx + radius <= 0 || my + radius <= 0) {
        return fallback;
      } else if (cx >= this.width - 1 || cy >= this.height - 1) {
        return fallback;
      }

      var i;
      var testLength = radius * 2 + 1;

      // Top.
      i = this.getFirstBlank(cx, cy, testLength);
      if (i >= 1) {
        return { x: i, y: cy };
      }

      i = this.getFirstBlank(cx, my + radius, testLength);
      if (i >= 1) {
        return { x: i, y: my + radius };
      }

      for (var ty = cy + 1; ty < my + radius; ty++) {
        if (cx > 0) {
          i = this.getFirstBlank(cx, ty, 1);
          if (i >= 1) {
            return { x: i, y: ty };
          }
        }

        if (mx + radius < this.width - 1) {
          i = this.getFirstBlank(mx + radius, ty, 1);
          if (i >= 1) {
            return { x: i, y: ty };
          }
        }
      }
    }

    return fallback;
  }

  toScreenX(mx: number): number {
    return mx * this.tilewidth + this.tilewidth / 2;
  }

  toScreenY(my: number): number {
    return my * this.tileheight + this.tileheight / 2;
  }

  toMapX(sx: number): number {
    return Math.floor(sx / this.tilewidth);
  }

  toMapY(sy: number): number {
    return Math.floor(sy / this.tileheight);
  }

  fixScreenX(sx: number): number {
    return this.toScreenX(this.toMapX(sx));
  }

  fixScreenY(sy: number): number {
    return this.toScreenY(this.toMapY(sy));
  }

  mapToChunkX(mx: number): number {
    return Math.floor(mx / this.chunkwidth);
  }

  mapToChunkY(my: number): number {
    return Math.floor(my / this.chunkheight);
  }

  screenToChunkX(sx: number): number {
    return this.mapToChunkX(this.toMapX(sx));
  }

  screenToChunkY(sy: number): number {
    return this.mapToChunkY(this.toMapY(sy));
  }
}
