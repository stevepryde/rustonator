import CellType from "./celltypes";

export interface IAgent {
  x: number;
  y: number;
  canPass(cellType: CellType): boolean;
}
