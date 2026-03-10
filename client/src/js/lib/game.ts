import Phaser from "phaser";
import { gameBridge } from "../../bridge/GameBridge";
import { Player, PlayerData } from "./common/player";
import { Action, ActionData } from "./common/action";
import { ChunkData, World, WorldData } from "./common/world";
import { ObjectPool } from "./objectpool";
import { Mob, MobData } from "./common/mob";
import { BombData } from "./common/bomb";
import { ExplosionData } from "./common/explosion";
import { EffectType } from "./common/effect";
import { PlayerFlags } from "./common/playerflags";
import { formatTimedStatusSummary } from "../../lib/playerStatus";
import { phaserAssets } from "../../assets/gameAssets";

export const targetFPS = 30;

export enum GameState {
    Menu = 0,
    InitGame = 1,
    FindServer = 2,
    Connected = 3,
    MainGame = 4,
    RespawnMenu = 5
}

interface CanvasInfo {
    width: number;
    height: number;
    canvasType: number;
}

interface TouchActions {
    [x: string]: boolean;
    up: boolean;
    down: boolean;
    left: boolean;
    right: boolean;
    bomb: boolean;
}

type DirectionKey = "up" | "down" | "left" | "right";

interface DirectionState {
    up: boolean;
    down: boolean;
    left: boolean;
    right: boolean;
}

interface KeyBindings {
    up: Phaser.Input.Keyboard.Key;
    down: Phaser.Input.Keyboard.Key;
    left: Phaser.Input.Keyboard.Key;
    right: Phaser.Input.Keyboard.Key;
    fire: Phaser.Input.Keyboard.Key;
    special: Phaser.Input.Keyboard.Key;
}

export interface DeadData {
    reason: string;
}

export interface PowerupData {
    text: string;
}

interface InterpolationConfig {
    followRate: number;
    snapDistance: number;
}

interface InterpolationTarget extends InterpolationConfig {
    object: { x: number; y: number; active?: boolean };
    targetX: number;
    targetY: number;
}

//  The Google WebFont Loader will look for this object, so create it before loading the script.
let WebFontConfig = {
    //  The Google Fonts we want to load (specify as many as you like in the array)
    google: {
        families: ["Raleway"]
    }
};

/** Depth constants to replace Phaser CE group z-ordering. */
const DEPTH = {
    WORLD: -100,
    BOMB: 10,
    PLAYER: 20,
    CUR_PLAYER: 30,
    MOB: 35,
    EXPLOSION: 40,
    POWERUP: 50,
    SHADE: 90,
    CONTROLS: 100
};

const INTERPOLATION = {
    LOCAL_PLAYER: { followRate: 28, snapDistance: 96 },
    REMOTE_PLAYER: { followRate: 14, snapDistance: 96 },
    MOB: { followRate: 14, snapDistance: 96 },
    BOMB: { followRate: 18, snapDistance: 48 },
    EXPLOSION: { followRate: 18, snapDistance: 48 }
} as const;

export class DetonatorGame extends Phaser.Scene {
    playerName!: string;
    character!: string;
    phaserGame: Phaser.Game | null = null;
    canvasInfo!: CanvasInfo;
    parentContainer: HTMLDivElement | null = null;

    socket: WebSocket | null = null;

    lagCounter: number = 0;
    pingSent: boolean = false;

    world!: World;
    worldSprites: (Phaser.GameObjects.Image | null)[] = [];
    lastTX: number = 0;
    lastTY: number = 0;

    cameraset: boolean = false;
    totalPlayers: number = 0;
    curPlayer: Player | null = null;
    curAction: ActionData | null = null;
    isDead: boolean = false;
    deadCounter: number = targetFPS * 2; // wait for 2 seconds before exiting game.
    quitFlag: boolean = false;
    deadSprite: Phaser.GameObjects.Sprite | null = null;
    actionList: ActionData[] = [];
    nextActionID: number = 0;

    lastClientMS: number = 0;
    curClientMS: number = 0;
    clientElapsedMS: number = 0;
    minMS: number = 1000 / targetFPS;
    deadReason: string = "";
    touchActions!: TouchActions;

    mykeys: KeyBindings | null = null; // Main keys.
    altkeys: KeyBindings | null = null; // Alternative keys.
    fireflag: boolean = false;

    showGhost = false;
    tmpPlayer: Player | null = null;

    worldGroup: Phaser.GameObjects.Group | null = null;
    playerGroup: Phaser.GameObjects.Group | null = null;
    curPlayerGroup: Phaser.GameObjects.Group | null = null;
    mobGroup: Phaser.GameObjects.Group | null = null;
    bombGroup: Phaser.GameObjects.Group | null = null;
    explosionGroup: Phaser.GameObjects.Group | null = null;
    powerupGroup: Phaser.GameObjects.Group | null = null;
    shadeGroup: Phaser.GameObjects.Group | null = null;
    controlsGroup: Phaser.GameObjects.Group | null = null;
    controlSprites: { [x: string]: Phaser.GameObjects.Sprite } = {};
    playerSprites: { [x: string]: Phaser.GameObjects.Sprite } = {};
    mobSprites: { [x: string]: Phaser.GameObjects.Sprite } = {};
    bombSprites: { [x: string]: Phaser.GameObjects.Sprite } = {};
    explosionEmitters: { [x: string]: Phaser.GameObjects.Particles.ParticleEmitter } = {};
    powerupSprites: Phaser.GameObjects.Text[] = [];
    playerNames: { [x: string]: Phaser.GameObjects.Text } = {};
    interpolationTargets: { [x: string]: InterpolationTarget } = {};
    playerSpriteServer: Phaser.GameObjects.Sprite | null = null;
    knownPlayers = new ObjectPool<Player>();
    knownMobs = new ObjectPool<Mob>();
    knownBombs = new ObjectPool<BombData>();
    knownExplosions = new ObjectPool<ExplosionData>();
    flickerToggle = false;
    flickerTimeout = 2;
    flickerCount = 0;

    touchEnabled: boolean = false;
    directionPressOrder: Record<DirectionKey, number> = {
        up: 0,
        down: 0,
        left: 0,
        right: 0
    };
    previousDirectionState: DirectionState = {
        up: false,
        down: false,
        left: false,
        right: false
    };
    directionSequence: number = 0;
    leaderboardNames: Phaser.GameObjects.Text[] = [];
    leaderboardScores: Phaser.GameObjects.Text[] = [];
    leaderboardShade: Phaser.GameObjects.Image | null = null;
    dyingSprites: Phaser.GameObjects.Sprite[] = [];
    scoreShade: Phaser.GameObjects.Image | null = null;
    scoreText: Phaser.GameObjects.Text | null = null;

    constructor(playerName: string, character: string, parentContainer: HTMLDivElement) {
        super({ key: "DetonatorGame" });

        this.playerName = playerName;
        this.character = character;
        this.parentContainer = parentContainer;
        this.canvasInfo = this.getCanvasInfo();

        this.world = new World();
        this.touchActions = {
            up: false,
            down: false,
            left: false,
            right: false,
            bomb: false
        };
    }

    quitGame(): void {
        gameBridge.emit("playerDied", {
            reason: this.deadReason,
            finalScore: this.curPlayer?.score ?? 0,
        });
        gameBridge.emit("screenChange", { screen: "postGame" });
    }

    startGame() {
        // Remove all child elements of canvas container - phaser bug!
        if (this.parentContainer) {
            while (this.parentContainer.hasChildNodes()) {
                let lastChild = this.parentContainer.lastChild;
                if (lastChild) {
                    this.parentContainer.removeChild(lastChild);
                }
            }
        }

        const config: Phaser.Types.Core.GameConfig = {
            type: this.canvasInfo.canvasType,
            width: this.canvasInfo.width,
            height: this.canvasInfo.height,
            parent: this.parentContainer ?? undefined,
            pixelArt: false,
            backgroundColor: "#000000",
            scene: this,
            scale: {
                mode: Phaser.Scale.FIT,
                autoCenter: Phaser.Scale.NO_CENTER
            },
            input: {
                keyboard: true,
                touch: true
            }
        };

        this.phaserGame = new Phaser.Game(config);
    }

    getCanvasInfo(): CanvasInfo {
        return {
            width: 800,
            height: 600,
            canvasType: Phaser.AUTO
        };
    }

    cleanup(): void {
        if (this.socket) {
            this.socket.onclose = null;
            this.socket.onmessage = null;
            this.socket.close();
            this.socket = null;
        }

        this.cameraset = false;
        this.mykeys = null;
        this.altkeys = null;
        this.curAction = null;
        this.curPlayer = null;

        if (this.worldGroup) {
            this.worldGroup.destroy(true);
            this.worldGroup = null;
        }

        if (this.playerGroup) {
            this.playerGroup.destroy(true);
            this.playerGroup = null;
        }

        if (this.curPlayerGroup) {
            this.curPlayerGroup.destroy(true);
            this.curPlayerGroup = null;
        }

        if (this.mobGroup) {
            this.mobGroup.destroy(true);
            this.mobGroup = null;
        }

        if (this.bombGroup) {
            this.bombGroup.destroy(true);
            this.bombGroup = null;
        }

        if (this.explosionGroup) {
            this.explosionGroup.destroy(true);
            this.explosionGroup = null;
        }

        if (this.powerupGroup) {
            this.powerupGroup.destroy(true);
            this.powerupGroup = null;
        }

        if (this.controlsGroup) {
            this.controlsGroup.destroy(true);
            this.controlsGroup = null;
        }

        if (this.shadeGroup) {
            this.shadeGroup.destroy(true);
            this.shadeGroup = null;
        }

        this.knownPlayers.clear();
        this.knownMobs.clear();
        this.playerSprites = {};
        this.mobSprites = {};
        this.controlSprites = {};

        this.knownBombs.clear();
        this.bombSprites = {};

        this.knownExplosions.clear();
        this.explosionEmitters = {};
        this.interpolationTargets = {};

        this.worldSprites = [];
        this.playerSpriteServer = null;
        this.leaderboardShade = null;
        this.scoreShade = null;
        this.scoreText = null;
        this.leaderboardNames = [];
        this.leaderboardScores = [];
        this.dyingSprites = [];
        this.directionPressOrder = {
            up: 0,
            down: 0,
            left: 0,
            right: 0
        };
        this.previousDirectionState = {
            up: false,
            down: false,
            left: false,
            right: false
        };
        this.directionSequence = 0;

        if (this.phaserGame) {
            this.phaserGame.destroy(true);
        }
    }

    preload(): void {
        this.load.spritesheet("p1", phaserAssets.p1, { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("p2", phaserAssets.p2, { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("p3", phaserAssets.p3, { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("p4", phaserAssets.p4, { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("mob1", phaserAssets.mob1, { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("tiles", phaserAssets.tiles, { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("explode", phaserAssets.explode, { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("bombs", phaserAssets.bombs, { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("controls", phaserAssets.controls, { frameWidth: 32, frameHeight: 32 });
        this.load.image("shade", phaserAssets.shade);

        this.load.script(
            "webfont",
            "https://ajax.googleapis.com/ajax/libs/webfont/1.4.7/webfont.js"
        );
    }

    isMobile(): boolean {
        return /Android|webOS|iPhone|iPad|iPod|BlackBerry|BB|PlayBook|IEMobile|Windows Phone|Kindle|Silk|Opera Mini/i.test(
            navigator.userAgent
        );
    }

    isAndroid(): boolean {
        return /Android/i.test(navigator.userAgent);
    }

    isApple(): boolean {
        return /(iPhone|iPad)/i.test(navigator.userAgent);
    }

    create(): void {
        // Create shared animations for player spritesheets.
        const playerKeys = ["p1", "p2", "p3", "p4"];
        for (const key of playerKeys) {
            if (!this.anims.exists(key + "_down")) {
                this.anims.create({
                    key: key + "_down",
                    frames: this.anims.generateFrameNumbers(key, { frames: [0, 1, 2, 1] }),
                    frameRate: 10,
                    repeat: -1
                });
                this.anims.create({
                    key: key + "_up",
                    frames: this.anims.generateFrameNumbers(key, { frames: [3, 4, 5, 4] }),
                    frameRate: 10,
                    repeat: -1
                });
                this.anims.create({
                    key: key + "_left",
                    frames: this.anims.generateFrameNumbers(key, { frames: [6, 7, 9, 7, 6, 7, 8, 7] }),
                    frameRate: 10,
                    repeat: -1
                });
                this.anims.create({
                    key: key + "_right",
                    frames: this.anims.generateFrameNumbers(key, { frames: [6, 7, 9, 7, 6, 7, 8, 7] }),
                    frameRate: 10,
                    repeat: -1
                });
            }
        }

        // Create shared animations for mob spritesheets.
        if (!this.anims.exists("mob1_updown")) {
            this.anims.create({
                key: "mob1_updown",
                frames: this.anims.generateFrameNumbers("mob1", { frames: [0, 1] }),
                frameRate: 10,
                repeat: -1
            });
            this.anims.create({
                key: "mob1_left",
                frames: this.anims.generateFrameNumbers("mob1", { frames: [2, 3] }),
                frameRate: 10,
                repeat: -1
            });
            this.anims.create({
                key: "mob1_right",
                frames: this.anims.generateFrameNumbers("mob1", { frames: [2, 3] }),
                frameRate: 10,
                repeat: -1
            });
        }

        // Set up keyboard input.
        if (this.input.keyboard) {
            this.mykeys = this.input.keyboard.addKeys({
                up: Phaser.Input.Keyboard.KeyCodes.UP,
                down: Phaser.Input.Keyboard.KeyCodes.DOWN,
                left: Phaser.Input.Keyboard.KeyCodes.LEFT,
                right: Phaser.Input.Keyboard.KeyCodes.RIGHT,
                fire: Phaser.Input.Keyboard.KeyCodes.SPACE,
                special: Phaser.Input.Keyboard.KeyCodes.COMMA
            }) as KeyBindings;
            this.altkeys = this.input.keyboard.addKeys({
                up: Phaser.Input.Keyboard.KeyCodes.W,
                down: Phaser.Input.Keyboard.KeyCodes.S,
                left: Phaser.Input.Keyboard.KeyCodes.A,
                right: Phaser.Input.Keyboard.KeyCodes.D,
                fire: Phaser.Input.Keyboard.KeyCodes.CTRL,
                special: Phaser.Input.Keyboard.KeyCodes.ALT
            }) as KeyBindings;
        }

        this.curAction = new Action().toJSON();

        this.worldGroup = this.add.group();
        this.bombGroup = this.add.group();
        this.playerGroup = this.add.group();
        this.curPlayerGroup = this.add.group();
        this.mobGroup = this.add.group();
        this.explosionGroup = this.add.group();
        this.powerupGroup = this.add.group();
        this.shadeGroup = this.add.group();
        this.controlsGroup = this.add.group();

        // Disable visibility change pausing.
        this.sys.game.events.on("hidden", () => { /* do nothing */ });
        this.sys.game.events.on("visible", () => { /* do nothing */ });

        if (this.sys.game.device.os.desktop) {
            this.touchEnabled = false;
        } else {
            this.touchEnabled = true;
            this.addTouchControls();
            this.leaderboardShade = null;
            this.scoreShade = null;
        }

        gameBridge.emit("screenChange", { screen: "game" });
    }

    update(time: number, delta: number): void {
        if (this.quitFlag) {
            return;
        }

        this.curClientMS = time;
        if (this.lastClientMS === 0) {
            this.lastClientMS = this.curClientMS - 1000 / targetFPS;
        }

        this.clientElapsedMS = this.curClientMS - this.lastClientMS;

        if (!this.cameraset && this.curPlayer && this.curPlayer.id in this.playerSprites) {
            this.cameras.main.startFollow(this.playerSprites[this.curPlayer.id]);
            this.cameraset = true;
        }

        if (!this.isDead) {
            // client-side prediction.
            this.clientSidePrediction();
        }

        this.handleTouch();
        this.updateInterpolatedObjects(delta / 1000);
        this.syncPlayerNames();
        this.updateBombVisuals(time);

        if (this.clientElapsedMS >= this.minMS) {

            this.lastClientMS = this.curClientMS;
            if (!this.curAction) {
                this.curAction = new Action().toJSON();
            }

            this.curAction.x = 0;
            this.curAction.y = 0;
            this.curAction.preferY = false;
            this.curAction.fire = false;
            this.curAction.deltaTime = 1.0 / targetFPS;
            if (this.mykeys && this.altkeys) {
                const rawDirectionState = this.getRawDirectionalState();
                this.updateDirectionPriority(rawDirectionState);
                const resolvedDirection = this.resolveDirectionState(this.getDirectionalState());
                this.curAction.x = resolvedDirection.x;
                this.curAction.y = resolvedDirection.y;
                this.curAction.preferY = resolvedDirection.preferY;
                if (this.mykeys.fire.isDown || this.altkeys.fire.isDown || this.touchActions["bomb"]) {
                    if (!this.fireflag) {
                        this.curAction.fire = true;

                        // Force separate presses each time.
                        this.fireflag = true;
                    }
                } else {
                    // not pressing fire.
                    this.fireflag = false;
                }
            }

            // Only send command to server if we're still alive.
            if (!this.isDead) {
                // If we're lagging badly - don't send any input :(
                // maximum 30 frames behind.
                if (this.actionList.length < targetFPS) {
                    this.curAction.id = this.nextActionID++;
                    this.socket_wrapper("ACTION", this.curAction);
                    this.actionList.push(Object.assign({}, this.curAction));
                }
            }
        }

        if (this.isDead) {
            // we're dead - so do the countdown.
            if (!this.deadSprite) {
                if (this.curPlayer) {
                    if (this.curPlayer.id in this.playerSprites) {
                        this.deadSprite = this.playerSprites[this.curPlayer.id];
                        this.deadSprite.stop();
                    } else {
                        this.deadSprite = this.add.sprite(
                            this.curPlayer.x,
                            this.curPlayer.y,
                            this.curPlayer.image,
                            1
                        );
                        this.deadSprite.setOrigin(0.5);
                        this.deadSprite.setDepth(DEPTH.CUR_PLAYER);
                        if (this.curPlayerGroup) {
                            this.curPlayerGroup.add(this.deadSprite);
                        }
                    }
                }
            }

            // Ghost float away...
            if (this.deadSprite) {
                this.deadSprite.y -= 1.5;
                if (this.deadSprite.alpha > 0.15) {
                    this.deadSprite.alpha -= 0.04;
                }
                // Gentle scale-up to ~1.3x, preserving flip direction.
                let sign = this.deadSprite.scaleX < 0 ? -1 : 1;
                if (Math.abs(this.deadSprite.scaleX) < 10.0) {
                    this.deadSprite.scaleX += sign * 0.05;
                    this.deadSprite.scaleY += 0.05;
                }
            }

            this.deadCounter--;
            if (this.deadCounter <= 0) {
                this.quitFlag = true;
                this.quitGame();
            }
        }

        // Animate dying other-player sprites.
        this.dyingSprites = this.dyingSprites.filter((sprite) => {
            sprite.y -= 1.5;
            sprite.alpha -= 0.04;
            let sign = sprite.scaleX < 0 ? -1 : 1;
            if (Math.abs(sprite.scaleX) < 10.0) {
                sprite.scaleX += sign * 0.05;
                sprite.scaleY += 0.05;
            }
            if (sprite.alpha <= 0.05) {
                sprite.destroy();
                return false;
            }
            return true;
        });
    }

    handleTouch(): void {
        this.touchActions = {
            up: false,
            down: false,
            left: false,
            right: false,
            bomb: false
        };

        if (this.input.pointer1) {
            this.checkPointer(this.input.pointer1);
        }
        if (this.input.pointer2) {
            this.checkPointer(this.input.pointer2);
        }
    }

    checkPointer(pointer: Phaser.Input.Pointer): void {
        if (!pointer.isDown) {
            return;
        }

        let controls = ["up", "down", "left", "right", "bomb"];
        for (let i = 0; i < controls.length; i++) {
            let actionlabel = controls[i];
            let sprite = this.controlSprites[actionlabel];
            if (sprite && this.spriteContains(sprite, pointer.x, pointer.y)) {
                this.touchActions[actionlabel] = true;
            }
        }
    }

    getDirectionalState(): DirectionState {
        const rawState = this.getRawDirectionalState();
        if (this.curPlayer?.hasEffect(EffectType.InputInversion)) {
            return {
                up: rawState.down,
                down: rawState.up,
                left: rawState.right,
                right: rawState.left
            };
        }

        return rawState;
    }

    getRawDirectionalState(): DirectionState {
        return {
            up: !!(this.mykeys?.up.isDown || this.altkeys?.up.isDown || this.touchActions.up),
            down: !!(this.mykeys?.down.isDown || this.altkeys?.down.isDown || this.touchActions.down),
            left: !!(this.mykeys?.left.isDown || this.altkeys?.left.isDown || this.touchActions.left),
            right: !!(this.mykeys?.right.isDown || this.altkeys?.right.isDown || this.touchActions.right)
        };
    }

    updateDirectionPriority(directionState: DirectionState): void {
        const directions: DirectionKey[] = ["up", "down", "left", "right"];
        for (let i = 0; i < directions.length; i++) {
            const direction = directions[i];
            if (directionState[direction] && !this.previousDirectionState[direction]) {
                this.directionPressOrder[direction] = ++this.directionSequence;
            }
            this.previousDirectionState[direction] = directionState[direction];
        }
    }

    resolveAxisDirection(
        negative: DirectionKey,
        positive: DirectionKey,
        directionState: DirectionState,
        inverted: boolean
    ): number {
        if (directionState[negative] === directionState[positive]) {
            if (!directionState[negative]) {
                return 0;
            }

            return this.getDirectionOrder(negative, inverted) > this.getDirectionOrder(positive, inverted)
                ? -1
                : 1;
        }

        return directionState[negative] ? -1 : 1;
    }

    resolveDirectionState(directionState: DirectionState): { x: number; y: number; preferY: boolean } {
        const inverted = !!this.curPlayer?.hasEffect(EffectType.InputInversion);
        const x = this.resolveAxisDirection("left", "right", directionState, inverted);
        const y = this.resolveAxisDirection("up", "down", directionState, inverted);

        if (x !== 0 && y !== 0) {
            const xOrder = this.getDirectionOrder(x < 0 ? "left" : "right", inverted);
            const yOrder = this.getDirectionOrder(y < 0 ? "up" : "down", inverted);
            return { x, y, preferY: yOrder > xOrder };
        }

        return { x, y, preferY: y !== 0 };
    }

    getDirectionOrder(direction: DirectionKey, inverted: boolean): number {
        if (!inverted) {
            return this.directionPressOrder[direction];
        }

        switch (direction) {
            case "up":
                return this.directionPressOrder.down;
            case "down":
                return this.directionPressOrder.up;
            case "left":
                return this.directionPressOrder.right;
            case "right":
                return this.directionPressOrder.left;
        }
    }

    spriteContains(sprite: Phaser.GameObjects.Sprite, x: number, y: number): boolean {
        // Control sprites have scrollFactor 0, so their screen position is their position directly.
        let sx = sprite.x;
        let sy = sprite.y;
        let sw = sprite.displayWidth;
        let sh = sprite.displayHeight;

        if (x >= sx && x < sx + sw && y >= sy && y < sy + sh) {
            return true;
        }

        return false;
    }

    goFull(): void {
        if (this.touchEnabled) {
            if (!this.scale.isFullscreen) {
                this.scale.startFullscreen();
            } else {
                this.scale.stopFullscreen();
            }
        }
    }

    addTouchControls(): void {
        if (!this.controlsGroup) {
            return;
        }

        // Add controls.
        let bottom = this.canvasInfo.height;
        let right = this.canvasInfo.width;
        let gap = 0;
        let scale = 2.5;
        let size = 32 * scale;
        let controlsx = 10;
        let controlsy = bottom - (10 + size + gap + size + gap + size);

        // UP
        let sprite = this.add.sprite(controlsx + size + gap, controlsy, "controls", 0);
        this.controlSprites["up"] = sprite;
        sprite.setOrigin(0, 0);
        sprite.setScrollFactor(0);
        sprite.setScale(scale);
        sprite.alpha = 0.8;
        sprite.setDepth(DEPTH.CONTROLS);
        this.controlsGroup.add(sprite);

        // DOWN
        sprite = this.add.sprite(
            controlsx + size + gap,
            controlsy + size + gap + size + gap,
            "controls",
            1
        );
        sprite.setOrigin(0, 0);
        sprite.setScrollFactor(0);
        this.controlSprites["down"] = sprite;
        sprite.setInteractive();
        sprite.setScale(scale);
        sprite.alpha = 0.8;
        sprite.setDepth(DEPTH.CONTROLS);
        this.controlsGroup.add(sprite);

        // LEFT
        sprite = this.add.sprite(controlsx, controlsy + size + gap, "controls", 2);
        sprite.setOrigin(0, 0);
        sprite.setScrollFactor(0);
        this.controlSprites["left"] = sprite;
        sprite.setInteractive();
        sprite.setScale(scale);
        sprite.alpha = 0.8;
        sprite.setDepth(DEPTH.CONTROLS);
        this.controlsGroup.add(sprite);

        // RIGHT
        sprite = this.add.sprite(
            controlsx + size + gap + size + gap,
            controlsy + size + gap,
            "controls",
            3
        );
        sprite.setOrigin(0, 0);
        sprite.setScrollFactor(0);
        this.controlSprites["right"] = sprite;
        sprite.setInteractive();
        sprite.setScale(scale);
        sprite.alpha = 0.8;
        sprite.setDepth(DEPTH.CONTROLS);
        this.controlsGroup.add(sprite);

        // BOMB
        sprite = this.add.sprite(right - (10 + size), bottom - (10 + size), "controls", 4);
        sprite.setOrigin(0, 0);
        sprite.setScrollFactor(0);
        this.controlSprites["bomb"] = sprite;
        sprite.setInteractive();
        sprite.setScale(scale);
        sprite.alpha = 0.8;
        sprite.setDepth(DEPTH.CONTROLS);
        this.controlsGroup.add(sprite);

        // Don't show fullscreen button on Android - point users to app instead.
        // Don't show the fullscreen button on iPhone/iPad either - it doesn't work.
        if (!this.isAndroid() && !this.isApple()) {
            // FULLSCREEN TOGGLE
            sprite = this.add.sprite(10, 10, "controls", 5);
            sprite.setOrigin(0, 0);
            sprite.setScrollFactor(0);
            this.controlSprites["fs"] = sprite;
            sprite.setInteractive();
            sprite.on("pointerdown", () => { this.goFull(); });
            sprite.setScale(2);
            sprite.alpha = 0.8;
            sprite.setDepth(DEPTH.CONTROLS);
            this.controlsGroup.add(sprite);
        }
    }

    socket_wrapper(cmd: string, data: any): void {
        // Override in subclasses.
    }

    spawnPlayer(player: PlayerData): void {
        // Use Player object to get defaults.
        this.tmpPlayer = new Player();
        this.curPlayer = new Player();
        this.curPlayer.fromJSON(player);
    }

    playerDied(data: string): void {
        this.isDead = true;
        this.deadReason = data;
        this.cameras.main.stopFollow();
    }

    updateStatus(): void {
        if (!this.curPlayer) {
            return;
        }

        // Mobile: show score in Phaser text overlay
        if (this.touchEnabled) {
            let status = "SCORE: " + this.curPlayer.score;
            status += "   BOMBS: " + this.curPlayer.maxBombs;
            status += "   RANGE: " + this.curPlayer.range;
            if (this.curPlayer.rank && this.totalPlayers) {
                status += "   RANK: " + this.curPlayer.rank + " of " + this.totalPlayers;
            }
            const timedStatuses = formatTimedStatusSummary(this.curPlayer);
            if (timedStatuses) {
                status += "\n" + timedStatuses;
            }

            if (!this.scoreText) {
                let text = this.add.text(310, this.canvasInfo.height - 20, status, {
                    fontFamily: "Raleway",
                    fontSize: "12px",
                    color: "#ffffff",
                    align: "left"
                });
                text.setScrollFactor(0);
                text.setOrigin(0);
                text.alpha = 0.8;
                text.setStroke("", 0);
                text.setShadow(1, 1, "rgba(0, 0, 0, 0.5)", 0);
                text.setDepth(DEPTH.CONTROLS);
                if (this.controlsGroup) {
                    this.controlsGroup.add(text);
                }
                this.scoreText = text;
            } else {
                this.scoreText.text = status;
            }

            if (!this.scoreShade) {
                this.scoreShade = this.add.image(300, this.canvasInfo.height - 25, "shade");
                this.scoreShade.setScrollFactor(0);
                this.scoreShade.setOrigin(0);
                this.scoreShade.setDepth(DEPTH.SHADE);
                if (this.shadeGroup) {
                    this.shadeGroup.add(this.scoreShade);
                }
            }

            this.scoreShade.displayWidth = this.scoreText.width + 20;
            this.scoreShade.displayHeight = this.scoreText.height + 8;

            return;
        }

        // Desktop: emit stats to React
        gameBridge.emit("playerStatsUpdate", {
            player: this.curPlayer.toJSON(),
            totalPlayers: this.totalPlayers,
        });
    }

    destroyPlayerSprite(pid: string): void {
        this.removeInterpolationTarget("player:" + pid);
        if (pid in this.playerSprites) {
            let sprite = this.playerSprites[pid];
            sprite.stop();
            this.dyingSprites.push(sprite);
            delete this.playerSprites[pid];
        }
    }

    destroyMobSprite(mid: string): void {
        this.removeInterpolationTarget("mob:" + mid);
        if (mid in this.mobSprites) {
            this.mobSprites[mid].setActive(false).setVisible(false);
            this.mobSprites[mid].destroy();
            delete this.mobSprites[mid];
        }
    }

    destroyPlayerName(pid: string): void {
        if (pid in this.playerNames) {
            this.playerNames[pid].setActive(false).setVisible(false);
            this.playerNames[pid].destroy();
            delete this.playerNames[pid];
        }
    }

    destroyBombSprite(bid: string): void {
        this.removeInterpolationTarget("bomb:" + bid);
        if (bid in this.bombSprites) {
            this.bombSprites[bid].setActive(false).setVisible(false);
            this.bombSprites[bid].destroy();
            delete this.bombSprites[bid];
        }
    }

    destroyExplosion(eid: string): void {
        this.removeInterpolationTarget("explosion:" + eid);
        if (eid in this.explosionEmitters) {
            this.explosionEmitters[eid].stop();
            this.explosionEmitters[eid].destroy();
            delete this.explosionEmitters[eid];
        }
    }

    createWorld(data: WorldData): void {
        this.world.fromJSON(data);

        this.worldSprites = [];
        for (let i = 0; i < this.world.width * this.world.height; i++) {
            this.worldSprites.push(null);
        }

        this.cameras.main.setBounds(
            0,
            0,
            this.world.width * this.world.tilewidth,
            this.world.height * this.world.tileheight
        );
    }

    updateWorld(data: ChunkData): void {
        let tx = data.tx;
        let ty = data.ty;
        let chunkwidth = data.width;
        let chunkheight = data.height;
        let mapdata = data.data;
        let index;
        let realIndex;
        let val;
        let mx, my;
        let tile: Phaser.GameObjects.Image | null;

        // Kill sprites that are no longer visible.
        if (tx !== this.lastTX) {
            let startx;
            let endx;
            if (tx < this.lastTX) {
                startx = tx + chunkwidth;
                endx = this.lastTX + chunkwidth;
            } else {
                startx = this.lastTX;
                endx = tx;
            }

            // remove slice.
            for (mx = startx; mx < endx; mx++) {
                for (my = this.lastTY; my < this.lastTY + chunkheight; my++) {
                    realIndex = my * this.world.width + mx;
                    let sprite = this.worldSprites[realIndex];
                    if (sprite != null) {
                        sprite.setActive(false).setVisible(false);
                        this.worldSprites[realIndex] = null;
                    }
                }
            }
        }

        if (ty !== this.lastTY) {
            let starty;
            let endy;
            if (ty < this.lastTY) {
                starty = ty + chunkheight;
                endy = this.lastTY + chunkheight;
            } else {
                starty = this.lastTY;
                endy = ty;
            }

            // remove slice.
            for (my = starty; my < endy; my++) {
                realIndex = my * this.world.width + this.lastTX;
                for (mx = this.lastTX; mx < this.lastTX + chunkwidth; mx++) {
                    let sprite = this.worldSprites[realIndex];
                    if (sprite != null) {
                        sprite.setActive(false).setVisible(false);
                        this.worldSprites[realIndex] = null;
                    }

                    realIndex++;
                }
            }
        }

        this.lastTX = tx;
        this.lastTY = ty;

        index = 0;
        for (my = ty; my < ty + chunkheight; my++) {
            realIndex = my * this.world.width + tx;

            for (mx = tx; mx < tx + chunkwidth; mx++) {
                val = mapdata[index++];

                // Update local world data.
                this.world.setcell(mx, my, val);

                // Update image.
                tile = this.worldSprites[realIndex];

                if (tile == null) {
                    let reused = false;
                    if (this.worldGroup) {
                        const children = this.worldGroup.getChildren() as Phaser.GameObjects.Image[];
                        for (let c = 0; c < children.length; c++) {
                            if (!children[c].active) {
                                tile = children[c];
                                tile.setPosition(mx * this.world.tilewidth, my * this.world.tileheight);
                                tile.setFrame(val);
                                tile.setActive(true).setVisible(true);
                                reused = true;
                                break;
                            }
                        }
                    }

                    if (!reused) {
                        tile = this.add.image(
                            mx * this.world.tilewidth,
                            my * this.world.tileheight,
                            "tiles",
                            val
                        );
                        tile.setOrigin(0, 0);
                        tile.setDepth(DEPTH.WORLD);
                        if (this.worldGroup) {
                            this.worldGroup.add(tile);
                        }
                    }

                    this.worldSprites[realIndex] = tile;
                } else {
                    tile.setFrame(val);
                }

                realIndex++;
            }
        }
    }

    emitPowerup(data: string): void {
        if (!this.curPlayer) {
            return;
        }

        let first = data.charAt(0);
        let color: string;
        if (first === "+") {
            color = "#00ff00";
        } else if (first === "-") {
            color = "#ff0000";
        } else {
            color = "#ffffff";
        }

        let text = this.add.text(this.curPlayer.x, this.curPlayer.y, data, {
            fontFamily: "Raleway",
            fontSize: "12px",
            color: color,
            align: "center"
        });
        text.setOrigin(0.5);
        text.setStroke("", 0);
        text.setShadow(1, 1, "rgba(0,0,0,0.8)", 0);
        text.alpha = 1.0;
        text.setDepth(DEPTH.POWERUP);

        if (this.powerupGroup) {
            this.powerupGroup.add(text);
        }
        this.powerupSprites.push(text);
    }

    updateVisiblePlayers(
        players: PlayerData[],
        bombs: BombData[],
        explosions: ExplosionData[],
        worlddata: ChunkData,
        mobs: MobData[]
    ): void {
        let pid: string;
        let bid: string;
        let eid: string;
        let mid: string;
        let i;
        let sprite: Phaser.GameObjects.Sprite;

        this.totalPlayers = players.length;

        // Crude flicker counter - to easily make invincible players stand out.
        this.flickerCount++;
        if (this.flickerCount >= this.flickerTimeout) {
            this.flickerToggle = !this.flickerToggle;
            this.flickerCount = 0;
        }

        this.updateWorld(worlddata);
        let halftilewidth = this.world.tilewidth / 2.0;
        let halftileheight = this.world.tileheight / 2.0;

        // Blank slate.
        this.knownPlayers.mark();
        this.knownMobs.mark();
        this.knownBombs.mark();
        this.knownExplosions.mark();

        // Update all visible players.
        for (i = 0; i < players.length; i++) {
            if (!players[i].active) {
                continue;
            }

            pid = players[i].id;
            let kPlayer = this.knownPlayers.get(pid);
            if (!kPlayer) {
                kPlayer = new Player();
            }
            this.knownPlayers.set(pid, kPlayer);

            // Update details about this player.
            kPlayer.fromJSON(players[i]);

            if (pid in this.playerSprites) {
                // Update player data.
                if (this.curPlayer && pid === this.curPlayer.id) {
                    this.curPlayer.fromJSON(players[i]);

                    // curPlayer sprite will be updated independently during update();
                } else {
                    sprite = this.playerSprites[pid];
                    this.setInterpolationTarget(
                        "player:" + pid,
                        sprite,
                        kPlayer.x,
                        kPlayer.y,
                        INTERPOLATION.REMOTE_PLAYER
                    );
                    this.setSprite(
                        sprite,
                        this.resolveRenderAction(sprite.x, sprite.y, kPlayer.x, kPlayer.y, kPlayer.action),
                        kPlayer.image
                    );
                }

                // Invincibility? (skip if dead — death animation controls alpha)
                if (!this.isDead || !(this.curPlayer && pid === this.curPlayer.id)) {
                    if (kPlayer.hasFlag(PlayerFlags.Invincible) && this.flickerToggle) {
                        let remaining = 10.0;
                        for (let effect of kPlayer.effects) {
                            if (effect.effectType === EffectType.Invincibility) {
                                remaining = effect.remaining;
                                break;
                            }
                        }
                        this.playerSprites[pid].alpha = remaining < 5.0 ? 0.5 : 0.1;
                    } else {
                        this.playerSprites[pid].alpha = 1;
                    }
                }
            } else {
                // spawn new sprite for this player.
                if (!kPlayer.image) {
                    // default image.
                    kPlayer.image = "p1";
                }

                if (this.curPlayer && pid === this.curPlayer.id) {
                    this.curPlayer.fromJSON(players[i]);

                    // SHOW SERVER COPY.
                    if (this.showGhost) {
                        this.playerSpriteServer = this.add.sprite(kPlayer.x, kPlayer.y, kPlayer.image);
                        this.playerSpriteServer.setOrigin(0.5);
                        this.playerSpriteServer.setDepth(DEPTH.CUR_PLAYER);
                        if (this.curPlayerGroup) {
                            this.curPlayerGroup.add(this.playerSpriteServer);
                        }
                    }

                    sprite = this.add.sprite(kPlayer.x, kPlayer.y, kPlayer.image);
                    sprite.setDepth(DEPTH.CUR_PLAYER);
                    if (this.curPlayerGroup) {
                        this.curPlayerGroup.add(sprite);
                    }
                } else {
                    sprite = this.add.sprite(kPlayer.x, kPlayer.y, kPlayer.image);
                    sprite.setDepth(DEPTH.PLAYER);
                    if (this.playerGroup) {
                        this.playerGroup.add(sprite);
                    }

                    this.makePlayerName(kPlayer);
                }

                sprite.setOrigin(0.5);
                this.playerSprites[pid] = sprite;
                this.setInterpolationTarget(
                    "player:" + pid,
                    sprite,
                    kPlayer.x,
                    kPlayer.y,
                    pid === this.curPlayer?.id
                        ? INTERPOLATION.LOCAL_PLAYER
                        : INTERPOLATION.REMOTE_PLAYER
                );
            }
        }

        // Update all visible mobs.
        for (i = 0; i < mobs.length; i++) {
            if (!mobs[i].active) {
                continue;
            }

            mid = mobs[i].id.toString();
            let kMob = this.knownMobs.get(mid);
            if (!kMob) {
                kMob = new Mob();
            }
            this.knownMobs.set(mid, kMob);

            // Update details about this mob.
            kMob.fromJSON(mobs[i]);

            if (mid in this.mobSprites) {
                // Update mob data.
                sprite = this.mobSprites[mid];
                this.setInterpolationTarget("mob:" + mid, sprite, kMob.x, kMob.y, INTERPOLATION.MOB);
                this.setSprite(
                    sprite,
                    this.resolveRenderAction(sprite.x, sprite.y, kMob.x, kMob.y, kMob.action),
                    kMob.image
                );
            } else {
                // spawn new sprite for this mob.
                if (!kMob.image) {
                    // default image.
                    kMob.image = "mob1";
                }

                sprite = this.add.sprite(kMob.x, kMob.y, kMob.image);
                sprite.setDepth(DEPTH.MOB);
                if (this.mobGroup) {
                    this.mobGroup.add(sprite);
                }

                sprite.setOrigin(0.5);
                this.mobSprites[mid] = sprite;
                this.setInterpolationTarget("mob:" + mid, sprite, kMob.x, kMob.y, INTERPOLATION.MOB);
            }
        }

        // Update all visible bombs.
        for (i = 0; i < bombs.length; i++) {
            if (!bombs[i]) {
                continue;
            }

            if (bombs[i].remaining <= 0 || !bombs[i].active) {
                continue;
            }

            bid = bombs[i].id.toString();
            this.knownBombs.set(bid, bombs[i]);

            let bx = bombs[i].x * this.world.tilewidth + halftilewidth;
            let by = bombs[i].y * this.world.tileheight + halftileheight;

            if (bid in this.bombSprites) {
                this.setInterpolationTarget(
                    "bomb:" + bid,
                    this.bombSprites[bid],
                    bx,
                    by,
                    INTERPOLATION.BOMB
                );
            } else {
                let bomb = this.add.sprite(bx, by, "bombs");
                bomb.setDepth(DEPTH.BOMB);
                if (this.bombGroup) {
                    this.bombGroup.add(bomb);
                }

                if (bombs[i].remote) {
                    bomb.setFrame(3);
                } else {
                    let frames: number[] = [];
                    let secsRemaining = Math.floor(bombs[i].remaining);
                    if (secsRemaining > 4) {
                        secsRemaining = 4;
                    }

                    for (let n = 4 - secsRemaining; n < 4; n++) {
                        frames.push(n);
                    }

                    let animKey = "bomb_blow_" + bid;
                    if (!this.anims.exists(animKey)) {
                        this.anims.create({
                            key: animKey,
                            frames: this.anims.generateFrameNumbers("bombs", { frames: frames }),
                            frameRate: 1,
                            repeat: 0
                        });
                    }
                    bomb.play(animKey);
                }
                bomb.setOrigin(0.5);
                this.bombSprites[bid] = bomb;
                this.setInterpolationTarget("bomb:" + bid, bomb, bx, by, INTERPOLATION.BOMB);
            }
        }

        // Update all visible explosions.
        for (i = 0; i < explosions.length; i++) {
            if (!explosions[i]) {
                continue;
            }

            if (!explosions[i].active) {
                continue;
            }

            eid = explosions[i].id.toString();
            this.knownExplosions.set(eid, explosions[i]);

            let ex = explosions[i].x * this.world.tilewidth + halftilewidth;
            let ey = explosions[i].y * this.world.tileheight + halftileheight;

            if (eid in this.explosionEmitters) {
                this.setInterpolationTarget(
                    "explosion:" + eid,
                    this.explosionEmitters[eid],
                    ex,
                    ey,
                    INTERPOLATION.EXPLOSION
                );
            } else {
                let ms = Math.floor(explosions[i].remaining * 1200);

                let emitter = this.add.particles(ex, ey, "explode", {
                    frame: [0, 1, 2, 3, 4, 5],
                    lifespan: ms,
                    speed: { min: -32, max: 32 },
                    scale: { start: 1.0, end: 0.5 },
                    alpha: { start: 1, end: 0 },
                    rotate: { min: 0, max: 100 },
                    gravityY: 0,
                    quantity: 3,
                    emitting: false
                });
                emitter.setDepth(DEPTH.EXPLOSION);
                if (this.explosionGroup) {
                    this.explosionGroup.add(emitter);
                }

                emitter.explode(3);

                this.explosionEmitters[eid] = emitter;
                this.setInterpolationTarget(
                    "explosion:" + eid,
                    emitter,
                    ex,
                    ey,
                    INTERPOLATION.EXPLOSION
                );
            }
        }

        // Clean up.
        this.knownPlayers.cleanUp((pid) => {
            if (!this.curPlayer || pid !== this.curPlayer.id) {
                this.destroyPlayerSprite(pid);
                this.destroyPlayerName(pid);
            }
        });

        this.knownMobs.cleanUp((mid) => {
            this.destroyMobSprite(mid);
        });

        this.knownBombs.cleanUp((bid) => {
            this.destroyBombSprite(bid);
            let animKey = "bomb_blow_" + bid;
            if (this.anims.exists(animKey)) {
                this.anims.remove(animKey);
            }
        });

        this.knownExplosions.cleanUp((eid) => {
            this.destroyExplosion(eid);
        });

        // Update powerup sprites.
        for (i = 0; i < this.powerupSprites.length; i++) {
            this.powerupSprites[i].y -= 0.5;
            this.powerupSprites[i].alpha -= 0.02;
        }

        // Clean up sprites.
        this.powerupSprites = this.powerupSprites.filter((f) => {
            if (f.alpha < 0.1) {
                f.destroy();
                return false;
            }

            return true;
        });

        this.updateLeaderboard(players);
        this.updateStatus();
    }

    updateLeaderboard(players: PlayerData[]): void {
        // Emit to React for rendering
        gameBridge.emit("leaderboardUpdate", {
            players,
            currentPlayerId: this.curPlayer?.id ?? null,
        });
    }

    private escapeHtml(text: string): string {
        return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    }

    clientSidePrediction(): void {
        if (!this.curPlayer || !this.tmpPlayer) {
            return;
        }

        let pid = this.curPlayer.id;

        if (!(pid in this.playerSprites)) {
            return;
        }

        if (this.playerSpriteServer) {
            if (this.showGhost) {
                this.playerSpriteServer.visible = true;
                this.playerSpriteServer.x = this.curPlayer.x;
                this.playerSpriteServer.y = this.curPlayer.y;
                this.playerSpriteServer.alpha = 0.4;
                this.setSprite(this.playerSpriteServer, this.curPlayer.action, this.curPlayer.image);
            } else {
                this.playerSpriteServer.visible = false;
            }
        }

        // Start with last known player position.
        this.tmpPlayer.fromJSON(this.curPlayer.toJSON());

        // Remove actions that have already been processed.
        this.actionList = this.actionList.filter((f) => {
            return this.curPlayer && f.id > this.curPlayer.action.id;
        });

        // Replay client-side actions.
        for (let i = 0; i < this.actionList.length; i++) {
            this.tmpPlayer.action.fromJSON(this.actionList[i]);
            this.movePlayer(this.tmpPlayer);
        }

        this.setInterpolationTarget(
            "player:" + pid,
            this.playerSprites[pid],
            this.tmpPlayer.x,
            this.tmpPlayer.y,
            INTERPOLATION.LOCAL_PLAYER
        );

        // Play animation according to direction.
        this.setSprite(this.playerSprites[pid], this.tmpPlayer.action, this.tmpPlayer.image);
    }

    movePlayer(player: Player): void {
        // Move player.
        let mx = this.world.toMapX(player.x);
        let my = this.world.toMapY(player.y);
        let targetX = this.world.toScreenX(mx);
        let targetY = this.world.toScreenY(my);
        if (this.world.getcell(mx, my) === 1) {
            return;
        }

        // Prevent illegal moves.
        let tmpaction = {
            x: player.action.x,
            y: player.action.y,
            preferY: !!player.action.preferY,
            deltaTime: player.action.deltaTime,
            fire: false,
            id: 0
        };

        this.fixPositionAndTmpAction(player, tmpaction, mx, my, targetX, targetY);

        // Lock to gridlines.
        let tolerance = this.world.tilewidth * 0.3;
        if (tmpaction.preferY) {
            if (tmpaction.y !== 0) {
                if (targetX > player.x + tolerance) {
                    tmpaction.y = 0;
                    tmpaction.x = 1;
                } else if (targetX < player.x - tolerance) {
                    tmpaction.y = 0;
                    tmpaction.x = -1;
                } else {
                    player.x = targetX;
                    tmpaction.x = 0;
                }
            } else if (tmpaction.x !== 0) {
                if (targetY > player.y + tolerance) {
                    tmpaction.x = 0;
                    tmpaction.y = 1;
                } else if (targetY < player.y - tolerance) {
                    tmpaction.x = 0;
                    tmpaction.y = -1;
                } else {
                    player.y = targetY;
                    tmpaction.y = 0;
                }
            }
        } else if (tmpaction.x !== 0) {
            if (targetY > player.y + tolerance) {
                tmpaction.x = 0;
                tmpaction.y = 1;
            } else if (targetY < player.y - tolerance) {
                tmpaction.x = 0;
                tmpaction.y = -1;
            } else {
                player.y = targetY;
                tmpaction.y = 0;
            }
        } else if (tmpaction.y !== 0) {
            if (targetX > player.x + tolerance) {
                tmpaction.y = 0;
                tmpaction.x = 1;
            } else if (targetX < player.x - tolerance) {
                tmpaction.y = 0;
                tmpaction.x = -1;
            } else {
                player.x = targetX;
                tmpaction.x = 0;
            }
        }

        player.updateWithTempAction(tmpaction, tmpaction.deltaTime);

        this.fixPositionAndTmpAction(player, tmpaction, mx, my, targetX, targetY);
    }

    fixPositionAndTmpAction(player: Player, tmpaction: ActionData, mx: number, my: number, targetX: number, targetY: number) {
        if (tmpaction.x !== 0 && !player.canPass(this.world.getcell(mx + tmpaction.x, my))) {
            if ((tmpaction.x < 0 && player.x <= targetX) ||
                (tmpaction.x > 0 && player.x >= targetX)) {
                tmpaction.x = 0;
                player.x = targetX;
            }
        }
        if (tmpaction.y !== 0 && !player.canPass(this.world.getcell(mx, my + tmpaction.y))) {
            if ((tmpaction.y < 0 && player.y <= targetY) ||
                (tmpaction.y > 0 && player.y >= targetY)) {
                tmpaction.y = 0;
                player.y = targetY;
            }
        }
    }

    setInterpolationTarget(
        key: string,
        object: { x: number; y: number; active?: boolean },
        targetX: number,
        targetY: number,
        config: InterpolationConfig
    ): void {
        let target = this.interpolationTargets[key];
        if (!target || target.object !== object) {
            this.interpolationTargets[key] = {
                object,
                targetX,
                targetY,
                followRate: config.followRate,
                snapDistance: config.snapDistance
            };
            object.x = targetX;
            object.y = targetY;
            return;
        }

        target.followRate = config.followRate;
        target.snapDistance = config.snapDistance;

        if (Phaser.Math.Distance.Between(object.x, object.y, targetX, targetY) > target.snapDistance) {
            object.x = targetX;
            object.y = targetY;
        }

        target.targetX = targetX;
        target.targetY = targetY;
    }

    removeInterpolationTarget(key: string): void {
        delete this.interpolationTargets[key];
    }

    updateInterpolatedObjects(deltaSeconds: number): void {
        const keys = Object.keys(this.interpolationTargets);
        for (let i = 0; i < keys.length; i++) {
            const key = keys[i];
            const target = this.interpolationTargets[key];
            const object = target.object;

            if (object.active === false) {
                delete this.interpolationTargets[key];
                continue;
            }

            const alpha = 1 - Math.exp(-target.followRate * deltaSeconds);
            object.x = Phaser.Math.Linear(object.x, target.targetX, alpha);
            object.y = Phaser.Math.Linear(object.y, target.targetY, alpha);

            if (Math.abs(object.x - target.targetX) < 0.1) {
                object.x = target.targetX;
            }
            if (Math.abs(object.y - target.targetY) < 0.1) {
                object.y = target.targetY;
            }
        }
    }

    updateBombVisuals(time: number): void {
        const bombIds = Object.keys(this.bombSprites);
        for (let i = 0; i < bombIds.length; i++) {
            const bid = bombIds[i];
            const sprite = this.bombSprites[bid];
            const bomb = this.knownBombs.get(bid);
            if (!sprite || !bomb) {
                continue;
            }

            if (bomb.remote) {
                sprite.setFrame(3);
                sprite.setTint(0xff7a5c);
                const pulse = 1.06 + Math.sin(time / 120) * 0.08;
                sprite.setScale(pulse);
            } else {
                sprite.clearTint();
                sprite.setScale(1);
            }
        }
    }

    syncPlayerNames(): void {
        const playerIds = Object.keys(this.playerNames);
        for (let i = 0; i < playerIds.length; i++) {
            const pid = playerIds[i];
            const sprite = this.playerSprites[pid];
            const text = this.playerNames[pid];
            if (!sprite || !text) {
                continue;
            }

            text.x = sprite.x;
            text.y = sprite.y - 20;
        }
    }

    resolveRenderAction(
        currentX: number,
        currentY: number,
        targetX: number,
        targetY: number,
        fallback: ActionData
    ): ActionData {
        if (fallback.x !== 0 || fallback.y !== 0) {
            return fallback;
        }

        const dx = targetX - currentX;
        const dy = targetY - currentY;
        if (Math.abs(dx) < 0.5 && Math.abs(dy) < 0.5) {
            return fallback;
        }

        if (Math.abs(dx) >= Math.abs(dy)) {
            return { ...fallback, x: dx > 0 ? 1 : -1, y: 0 };
        }

        return { ...fallback, x: 0, y: dy > 0 ? 1 : -1 };
    }

    setSprite(sprite: Phaser.GameObjects.Sprite, action: ActionData, imageKey: string): void {
        if (action.x !== 0 || action.y !== 0) {
            let anim: string = "";

            if (action.preferY && action.y < 0) {
                anim = "up";
            } else if (action.preferY && action.y > 0) {
                anim = "down";
            } else if (action.x < 0) {
                anim = "left";
                sprite.scaleX = -1;
            } else if (action.x > 0) {
                anim = "right";
                sprite.scaleX = 1;
            } else if (action.y < 0) {
                anim = "up";
            } else if (action.y > 0) {
                anim = "down";
            }

            if (anim && anim.length > 0) {
                let animKey = imageKey + "_" + anim;
                let curAnimKey = sprite.anims.currentAnim ? sprite.anims.currentAnim.key : null;
                if (curAnimKey !== animKey || !sprite.anims.isPlaying) {
                    sprite.play(animKey);
                }
            }
        } else {
            sprite.anims.stop();
        }
    }

    makePlayerName(player: Player): boolean {
        if (player.id in this.playerNames) {
            return false;
        }

        if (!player.name) {
            return false;
        }

        let text = this.add.text(player.x, player.y - 20, player.name, {
            fontFamily: "Raleway",
            fontSize: "12px",
            color: "#ffffff",
            align: "center"
        });
        text.setOrigin(0.5);
        text.alpha = 0.8;
        text.setStroke("", 0);
        text.setShadow(1, 1, "rgba(0,0,0,0.5)", 0);
        text.setDepth(DEPTH.PLAYER);

        if (this.playerGroup) {
            this.playerGroup.add(text);
        }
        this.playerNames[player.id] = text;

        return true;
    }

    movePlayerName(player: Player): void {
        if (!(player.id in this.playerNames)) {
            if (!this.makePlayerName(player)) {
                return;
            }
        }

        this.playerNames[player.id].x = player.x;
        this.playerNames[player.id].y = player.y - 20;
    }
}
