import PlayerFlags from "./playerflags";

export interface EffectTarget {
    speed: number;
    addFlag(flag: PlayerFlags): void;
    delFlag(flag: PlayerFlags): void;
}

export enum EffectType {
    SpeedUp = 0,
    SlowDown = 1,
    Invincibility = 100
}

export function getRandomEffectType(): EffectType {
    const values = [0, 1];
    return values[Math.floor(Math.random() * values.length)];
}

export interface EffectData {
    effectType: EffectType;
    remaining: number;
}

export class Effect {
    effectType: EffectType;
    remaining: number;
    name: string;

    constructor(target: EffectTarget, etype: EffectType, duration: number) {
        this.effectType = etype;
        this.remaining = duration || 5; // Remaining seconds.
        this.name = "";

        this.createEffect(target);
    }

    toJSON(): EffectData {
        return {
            effectType: this.effectType,
            remaining: this.remaining
        };
    }

    fromJSON(data: EffectData) {
        this.effectType = data.effectType;
        this.remaining = data.remaining;
    }

    createEffect(target: EffectTarget): void {
        switch (this.effectType) {
            case EffectType.SpeedUp: // Speed up.
                target.speed += 50;
                this.name = ">>";
                break;
            case EffectType.SlowDown: // Slow down.
                target.speed -= 50;
                this.name = "<<";
                break;
            case EffectType.Invincibility: // Invincible.
                target.addFlag(PlayerFlags.Invincible);
                break;
            default:
                // Unknown effect - just kill it immediately.
                this.remaining = 0;
                this.effectType = -1;
        }
    }

    destroyEffect(target: EffectTarget): void {
        // Undo the effect that was added in createEffect().
        switch (this.effectType) {
            case EffectType.SpeedUp: // Speed up finished.
                target.speed -= 50;
                break;
            case EffectType.SlowDown: // Slow down finished.
                target.speed += 50;
                break;
            case EffectType.Invincibility: // end invincibility
                target.delFlag(2);
                break;
            default:
                // Unknown effect - just kill it immediately.
                this.remaining = 0;
                this.effectType = -1;
        }
    }

    update(deltaTime: number, target: EffectTarget): void {
        if (this.remaining <= 0) {
            // Already finished. Ignore it.
            return;
        }

        this.remaining -= deltaTime;

        if (this.remaining <= 0) {
            this.remaining = 0;

            this.destroyEffect(target);
        }
    }
}
