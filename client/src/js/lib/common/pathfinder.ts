import { World, PositionData } from "./world";
import { IAgent } from "./agent";

export interface PathFindData {
  mx: number;
  my: number;
  travelled: number;
  actionx: number;
  actiony: number;
  done: boolean;
  first?: boolean;
}

export type CanPassFunc = (x: number, y: number) => boolean;

export function defaultCanPassFunc(world: World, agent: IAgent) {
  return (x: number, y: number): boolean => {
    return agent.canPass(world.getcell(x, y));
  };
}

export class PathFinder {
  canPassFunc: CanPassFunc;

  constructor(func: CanPassFunc) {
    this.canPassFunc = func;
  }

  setCanPassFunc(func: CanPassFunc): void {
    this.canPassFunc = func;
  }

  addPossibleMoves(
    element: PathFindData,
    openList: PathFindData[],
    closedList: PathFindData[]
  ): boolean {
    let mx = element.mx;
    let my = element.my;
    let travelled = element.travelled + 1;
    let actions = [[0, -1], [1, 0], [0, 1], [-1, 0]];

    let added = false;

    for (let i = 0; i < actions.length; i++) {
      let xaction = actions[i][0];
      let yaction = actions[i][1];

      let cx = mx + xaction;
      let cy = my + yaction;

      let found = false;
      for (let ci = 0; ci < closedList.length; ci++) {
        if (closedList[ci].mx === cx && closedList[ci].my === cy) {
          found = true;
          break;
        }
      }

      if (found) {
        continue;
      }

      if (this.canPassFunc(cx, cy)) {
        let origax = element.actionx;
        let origay = element.actiony;
        if (element.first) {
          origax = xaction;
          origay = yaction;
        }

        openList.push({
          mx: cx,
          my: cy,
          travelled: travelled,
          actionx: origax,
          actiony: origay,
          done: false
        });
        added = true;
      }
    }

    return added;
  }

  // A* algorithm.
  pathFind(
    mx: number,
    my: number,
    targetmx: number,
    targetmy: number,
    maxdist: number
  ): PositionData | null {
    if (mx === targetmx && my === targetmy) {
      return null;
    }

    let openList: PathFindData[] = [];
    let closedList: PathFindData[] = [
      { mx: mx, my: my, travelled: 0, actionx: 0, actiony: 0, done: true }
    ];

    let initial = {
      mx: mx,
      my: my,
      travelled: 0,
      actionx: 0,
      actiony: 0,
      done: false,
      first: true
    };
    this.addPossibleMoves(initial, openList, closedList);

    while (openList.length) {
      // sort the list - the compare function uses the sum of the
      // tiles travelled and the distance to target.
      openList.sort(function(a, b) {
        let dxa = targetmx > a.mx ? targetmx - a.mx : a.mx - targetmx;
        let dya = targetmy > a.my ? targetmy - a.my : a.my - targetmy;

        let dxb = targetmx > b.mx ? targetmx - b.mx : b.mx - targetmx;
        let dyb = targetmy > b.my ? targetmy - b.my : b.my - targetmy;

        let dista = dxa + dya;
        let distb = dxb + dyb;

        // let dista = Math.sqrt(dxa * dxa + dya * dya);
        // let distb = Math.sqrt(dxb * dxb + dyb * dyb);
        //return (a.travelled + dista) - (b.travelled + distb);
        return dista - distb;
      });

      let element = openList[0];

      let processed = 0;
      for (let i = 0; i < openList.length; i++) {
        // Are we there yet?
        element = openList[i];
        if (element.mx === targetmx && element.my === targetmy) {
          return { x: element.actionx, y: element.actiony };
        }

        // add to closed list.
        closedList.push(element);

        processed = i;

        // if we've travelled too far, ignore it.
        if (element.travelled < maxdist) {
          // process this one.
          if (this.addPossibleMoves(element, openList, closedList)) {
            //we added a move. sort the list again before going
            // further.
            break;
          }
        }
      }

      openList.splice(0, processed + 1);
    }

    // NOTE: can't get there.
    return null;
  }
}
