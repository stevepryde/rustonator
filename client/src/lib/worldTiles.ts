import CellType from "../js/lib/common/celltypes";

export function getWorldTileFrame(cellType: number): number {
  if (cellType === CellType.Bomb) {
    return CellType.Empty;
  }

  return cellType;
}
