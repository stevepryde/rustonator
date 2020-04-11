import { Action, ActionData } from "./action";
import PlayerFlags from "./playerflags";
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
    flags: PlayerFlags;
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
    flags: PlayerFlags;
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
        this.flags = 0; // Player flags.
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
        this.action.fromJSON(data.action);
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

    updateWithTempAction(tmpaction: ActionData, deltaTime: number): void {
        // Process effects.
        if (this.effects.length > 0) {
            for (var i = 0; i < this.effects.length; i++) {
                this.effects[i].update(deltaTime, this);
            }

            // Remove finished effects.
            this.effects = this.effects.filter(function (f) {
                return f.remaining > 0;
            });
        }

        if (tmpaction) {
            var effectiveSpeed = this.speed;
            // NOTE: some effects may adjust speed outside safe limits.
            if (effectiveSpeed < 50) {
                effectiveSpeed = 50;
            } else if (effectiveSpeed > 300) {
                effectiveSpeed = 300;
            }

            // THIS could be a bug - don't use the deltaTime from an Action! this could be user-supplied!
            // On the other hand, client-side prediction might break if the precise delta_time was not used for each update.
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
        this.flags |= flag;
    }

    delFlag(flag: PlayerFlags): void {
        this.flags &= ~flag;
    }

    hasFlag(flag: PlayerFlags): boolean {
        return this.flags & flag ? true : false;
    }
}
