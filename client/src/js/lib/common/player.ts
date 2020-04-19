import { Action, ActionData } from "./action";
import { PlayerFlags } from "./playerflags";
import { Effect, EffectData, EffectTarget } from "./effect";
import CellType from "./celltypes";
import { IAgent } from "./agent";

export interface PlayerData {
    id: string;
    active: boolean;
    x: number;
    y: number;
    action: ActionData;
    speed: number;
    image: string;
    range: number;
    bombTime: number;
    maxBombs: number;
    curBombs: number;
    flags: string[];
    score: number;
    name: string;
    rank: number;
    effects: EffectData[];
}

export class Player implements EffectTarget, IAgent {
    id: string;
    active: boolean;
    x: number;
    y: number;
    action: Action;
    speed: number;
    image: string;
    range: number;
    bombTime: number;
    maxBombs: number;
    curBombs: number;
    flags: string[];
    score: number;
    name: string;
    rank: number;
    effects: Effect[];

    constructor() {
        this.id = "";
        this.active = true;
        this.x = 0;
        this.y = 0;
        this.action = new Action();
        this.speed = 200; // speed (in pixels per second).
        this.image = "p1"; //Image label.
        this.range = 1; // default bomb range (in each direction).
        this.bombTime = 3; // Seconds until bomb explodes. Max: 4, Min: 1.
        this.maxBombs = 1; // Bomb limit.
        this.curBombs = 0; // Number of bombs currently deployed.
        this.flags = []; // Player flags.
        this.score = 0;
        this.name = "";
        this.rank = 0;
        this.effects = [];
    }

    toJSON(): PlayerData {
        return {
            id: this.id,
            active: this.active,
            x: this.x,
            y: this.y,
            action: this.action.toJSON(),
            speed: this.speed,
            image: this.image,
            range: this.range,
            bombTime: this.bombTime,
            maxBombs: this.maxBombs,
            curBombs: this.curBombs,
            flags: this.flags,
            score: this.score,
            name: this.name,
            rank: this.rank,
            effects: this.effects.map((e: Effect) => {
                return e.toJSON();
            })
        };
    }

    fromJSON(data: PlayerData): void {
        this.id = data.id;
        this.active = data.active;
        this.x = data.x;
        this.y = data.y;

        // We need to fudge the action to make animations work smoothly.
        // The action will be cleared by the server if no input is registered for one frame,
        // but we don't want the client to clear animations just because it was late sending
        // input. However, client-side-prediction does need to clear the input to match the server.
        // This is a nasty hack - set the deltaTime to 0, which means CSP effectively drops
        // this input frame while the animation can still use the x and y to determine direction.
        let tmpAction = this.action;
        tmpAction.fromJSON(data.action);
        if (tmpAction.x != 0 || tmpAction.y != 0) {
            this.action = tmpAction;
        } else {
            this.action.deltaTime = 0;
        }
        this.speed = data.speed;
        this.image = data.image;
        this.range = data.range;
        this.bombTime = data.bombTime;
        this.maxBombs = data.maxBombs;
        this.curBombs = data.curBombs;
        this.flags = data.flags;
        this.score = data.score;
        this.name = data.name;
        this.rank = data.rank;
        this.effects = data.effects.map((e: EffectData) => {
            return new Effect(this, e.effectType, e.remaining);
        });
    }

    update(deltaTime: number): void {
        this.updateWithTempAction(this.action, deltaTime);
    }

    getEffectiveSpeed(): number {
        let effectiveSpeed = this.speed;
        // NOTE: some effects may adjust speed outside safe limits.
        if (effectiveSpeed < 50) {
            effectiveSpeed = 50;
        } else if (effectiveSpeed > 300) {
            effectiveSpeed = 300;
        }
        return effectiveSpeed;
    }

    updateWithTempAction(tmpaction: ActionData, deltaTime: number): void {
        // Process effects.
        if (this.effects.length > 0) {
            for (let i = 0; i < this.effects.length; i++) {
                this.effects[i].update(deltaTime, this);
            }

            // Remove finished effects.
            this.effects = this.effects.filter(function (f) {
                return f.remaining > 0;
            });
        }

        if (tmpaction) {
            let effectiveSpeed = this.getEffectiveSpeed();
            this.x += tmpaction.x * tmpaction.deltaTime * effectiveSpeed;
            this.y += tmpaction.y * tmpaction.deltaTime * effectiveSpeed;
        }
    }

    canPass(cellType: CellType): boolean {
        switch (cellType) {
            case CellType.Wall:
            case CellType.Mystery:
                return false;
            case CellType.Bomb:
                // Can the player walk through bombs?
                return this.hasFlag(PlayerFlags.WalkThroughBombs);
            default:
                return true;
        }
    }

    addFlag(flag: PlayerFlags): void {
        this.flags.push(flag);
    }

    delFlag(flag: PlayerFlags): void {
        let index = this.flags.indexOf(flag);
        this.flags.splice(index, 1);
    }

    hasFlag(flag: PlayerFlags): boolean {
        return this.flags.includes(flag);
    }
}
