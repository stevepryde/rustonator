import Phaser from "phaser";
import { IMG_PREFIX, setElementDisplay, WebUIManager, withElement } from "./web";
import { Player, PlayerData } from "./common/player";
import { Action, ActionData } from "./common/action";
import { ChunkData, World, WorldData } from "./common/world";
import { ObjectPool } from "./objectpool";
import { Mob, MobData } from "./common/mob";
import { BombData } from "./common/bomb";
import { ExplosionData } from "./common/explosion";
import { StateMachine } from "./statemachine";
import { EffectType } from "./common/effect";
import { PlayerFlags } from "./common/playerflags";

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

export class DetonatorGame extends Phaser.Scene {
    uiManager!: WebUIManager;
    stateMachine!: StateMachine;
    playerName!: string;
    gameState: GameState = GameState.Menu;
    phaserGame: Phaser.Game | null = null;
    canvasInfo!: CanvasInfo;

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
    playerSpriteServer: Phaser.GameObjects.Sprite | null = null;
    knownPlayers = new ObjectPool<Player>();
    knownMobs = new ObjectPool<Mob>();
    knownBombs = new ObjectPool<BombData>();
    knownExplosions = new ObjectPool<ExplosionData>();
    flickerToggle = false;
    flickerTimeout = 2;
    flickerCount = 0;

    touchEnabled: boolean = false;
    playerStatsDisplayed: boolean = false;
    iconDisplayed: boolean = false;
    leaderboardNames: Phaser.GameObjects.Text[] = [];
    leaderboardScores: Phaser.GameObjects.Text[] = [];
    leaderboardShade: Phaser.GameObjects.Image | null = null;
    scoreShade: Phaser.GameObjects.Image | null = null;
    scoreText: Phaser.GameObjects.Text | null = null;

    constructor(uiManager: WebUIManager, playerName: string, stateMachine: StateMachine) {
        super({ key: "DetonatorGame" });

        this.uiManager = uiManager;
        this.stateMachine = stateMachine;
        this.playerName = playerName;
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
        this.stateMachine.setState(GameState.RespawnMenu, {
            message: this.deadReason,
            player: this.curPlayer
        });
    }

    startGame() {
        this.uiManager.hideExitPopup();

        // Remove all child elements of gameCanvas - phaser bug!
        withElement("gameCanvas", (canvas) => {
            while (canvas.hasChildNodes()) {
                let lastChild = canvas.lastChild;
                if (lastChild) {
                    canvas.removeChild(lastChild);
                }
            }
        });

        const config: Phaser.Types.Core.GameConfig = {
            type: this.canvasInfo.canvasType,
            width: this.canvasInfo.width,
            height: this.canvasInfo.height,
            parent: "gameCanvas",
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
        // TODO: This should be unnecessary. Just destroy and recreate this game object.
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

        this.worldSprites = [];
        this.playerSpriteServer = null;
        this.leaderboardShade = null;
        this.scoreShade = null;
        this.scoreText = null;
        this.leaderboardNames = [];
        this.leaderboardScores = [];

        if (this.phaserGame) {
            this.phaserGame.destroy(true);
        }
    }

    preload(): void {
        let prefix = IMG_PREFIX;

        this.load.spritesheet("p1", prefix + "p1.png", { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("p2", prefix + "p2.png", { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("p3", prefix + "p3.png", { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("p4", prefix + "p4.png", { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("mob1", prefix + "mob1.png", { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("tiles", prefix + "tileset1.png", { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("explode", prefix + "explode.png", { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("bombs", prefix + "bombtiles.png", { frameWidth: 32, frameHeight: 32 });
        this.load.spritesheet("controls", prefix + "controls.png", { frameWidth: 32, frameHeight: 32 });
        this.load.image("shade", prefix + "shade.png");

        this.load.script(
            "webfont",
            "https://ajax.googleapis.com/ajax/libs/webfont/1.4.7/webfont.js"
        );
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
            this.iconDisplayed = true;

            setElementDisplay({
                iconarea: "block",
                player: "block"
            });

            this.playerStatsDisplayed = true;
        } else {
            this.touchEnabled = true;

            // Handle orientation changes via resize event.
            this.scale.on("resize", () => {
                this.handleOrientationChange();
            });

            this.iconDisplayed = false;
            setElementDisplay({
                iconarea: "none",
                player: "none",
                leaderboard: "none"
            });

            this.playerStatsDisplayed = false;

            this.addTouchControls();
            this.leaderboardShade = null;
            this.scoreShade = null;
        }

        this.uiManager.showGame();
    }

    handleOrientationChange(): void {
        if (this.sys.game.device.os.desktop) {
            return;
        }

        if (window.innerWidth < window.innerHeight) {
            // Portrait mode - incorrect orientation.
            setElementDisplay({ orientwarning: "block" });
        } else {
            setElementDisplay({ orientwarning: "none" });
        }
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

        if (this.clientElapsedMS >= this.minMS) {

            this.lastClientMS = this.curClientMS;
            if (!this.curAction) {
                this.curAction = new Action().toJSON();
            }

            this.curAction.x = 0;
            this.curAction.y = 0;
            this.curAction.fire = false;
            this.curAction.deltaTime = 1.0 / targetFPS;
            if (this.mykeys && this.altkeys) {
                if (this.mykeys.left.isDown || this.altkeys.left.isDown || this.touchActions["left"]) {
                    this.curAction.x -= 1;
                }
                if (this.mykeys.right.isDown || this.altkeys.right.isDown || this.touchActions["right"]) {
                    this.curAction.x += 1;
                }
                if (this.mykeys.up.isDown || this.altkeys.up.isDown || this.touchActions["up"]) {
                    this.curAction.y -= 1;
                }
                if (this.mykeys.down.isDown || this.altkeys.down.isDown || this.touchActions["down"]) {
                    this.curAction.y += 1;
                }
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
        if (!this.uiManager.isAndroid() && !this.uiManager.isApple()) {
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
        } else if (this.uiManager.isApple()) {
            // EXIT BUTTON
            sprite = this.add.sprite(10, 10, "controls", 6);
            sprite.setOrigin(0, 0);
            sprite.setScrollFactor(0);
            this.controlSprites["exit"] = sprite;
            sprite.setInteractive();
            sprite.on("pointerdown", () => { this.uiManager.showExitPopup(); });
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
        let status: string;

        if (!this.curPlayer) {
            return;
        }

        if (this.touchEnabled) {
            status = "SCORE: " + this.curPlayer.score;
            status += "   BOMBS: " + this.curPlayer.maxBombs;
            status += "   RANGE: " + this.curPlayer.range;
            if (this.curPlayer.rank && this.totalPlayers) {
                status += "   RANK: " + this.curPlayer.rank + " of " + this.totalPlayers;
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
            this.scoreShade.displayHeight = 20;

            return;
        }

        // Only display if the window is big enough.
        let h = Math.max(document.documentElement.clientHeight, window.innerHeight || 0);

        let threshold = 850;
        if (h < threshold && this.playerStatsDisplayed) {
            setElementDisplay({ player: "none" });
            this.playerStatsDisplayed = false;
        } else if (h > threshold && !this.playerStatsDisplayed) {
            setElementDisplay({ player: "block" });
            this.playerStatsDisplayed = true;
        }

        if (this.playerStatsDisplayed) {
            status = "<div class='heading'>" + this.curPlayer.name + "</div><hr />";
            status += "<table border='0' width='100%'><tr>";
            status += "<td width='50%'><b>SCORE:</b> " + this.curPlayer.score + "</td>";
            status += "<td><b>LOCATION:</b> ";
            status += this.world.toMapX(this.curPlayer.x) + " , " + this.world.toMapY(this.curPlayer.y);
            status += "</td></tr><tr>";
            status += "<td><b>BOMBS:</b> " + this.curPlayer.maxBombs + "</td>";
            status += "<td>";
            if (this.curPlayer.rank && this.totalPlayers) {
                status += "<b>RANK:</b> " + this.curPlayer.rank + " of " + this.totalPlayers;
            } else {
                status += "&nbsp;";
            }
            status += "</td></tr><tr>";
            status += "<td colspan='2'><b>RANGE:</b> " + this.curPlayer.range;
            status += "</td></tr>";
            status += "</table>";

            withElement("player", (elem) => {
                elem.innerHTML = status;
            });
        }
    }

    updateIcon(): void {
        let w = Math.max(document.documentElement.clientWidth, window.innerWidth || 0);
        let h = Math.max(document.documentElement.clientHeight, window.innerHeight || 0);
        let aspect = w / h;
        let minAspect = (this.canvasInfo.width + 320) / this.canvasInfo.height; // allow 250 pixels for the icon.

        let threshold = 650;
        if ((h < threshold || aspect < minAspect) && this.iconDisplayed) {
            setElementDisplay({ iconarea: "none", player: "none" });
            withElement("leaderboard", (elem) => {
                elem.style.top = "30px";
                // shrink font size.
                elem.style.fontSize = "8px";
                elem.style.width = "200px";
            });

            this.iconDisplayed = false;
        } else if (h > threshold && aspect >= minAspect && !this.iconDisplayed) {
            setElementDisplay({ iconarea: "block" });
            withElement("leaderboard", (elem) => {
                elem.style.top = "300px";
                elem.style.width = "400px";
            });
            withElement("player", (elem) => {
                elem.style.display = "block";
                elem.style.top = "600px";
            });
            this.iconDisplayed = true;
        }
    }

    destroyPlayerSprite(pid: string): void {
        if (pid in this.playerSprites) {
            this.playerSprites[pid].setActive(false).setVisible(false);
            this.playerSprites[pid].destroy();
            delete this.playerSprites[pid];
        }
    }

    destroyMobSprite(mid: string): void {
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
        if (bid in this.bombSprites) {
            this.bombSprites[bid].setActive(false).setVisible(false);
            this.bombSprites[bid].destroy();
            delete this.bombSprites[bid];
        }
    }

    destroyExplosion(eid: string): void {
        if (eid in this.explosionEmitters) {
            this.explosionEmitters[eid].stop();
            this.explosionEmitters[eid].destroy();
            delete this.explosionEmitters[eid];
        }
    }

    createWorld(data: WorldData): void {
        this.world.fromJSON(data);

        // Custom world using sprites.
        // NOTE: I tried using tilemap and it was horribly slow.
        //       Also there was no easy way to update a chunk of the map without
        //       reloading the whole thing, or using putTile() which crashed
        //       the browser tab because it was so slow!
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

                        // Now make it null. The group will keep track of the
                        // original object.
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

                        // Now make it null. The group will keep track of the
                        // original object.
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
            // Find the corresponding 'real' map index.
            realIndex = my * this.world.width + tx;

            for (mx = tx; mx < tx + chunkwidth; mx++) {
                val = mapdata[index++];

                // Update local world data.
                this.world.setcell(mx, my, val);

                // Update image.
                tile = this.worldSprites[realIndex];

                if (tile == null) {
                    // Try to find an inactive sprite in the world group to reuse.
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
                    this.playerSprites[pid].x = kPlayer.x;
                    this.playerSprites[pid].y = kPlayer.y;

                    this.setSprite(this.playerSprites[pid], kPlayer.action, kPlayer.image);

                    this.movePlayerName(kPlayer);
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

                sprite.x = kMob.x;
                sprite.y = kMob.y;

                let mobAction = mobs[i].action;
                let anim = "updown";

                if (mobAction.x < 0) {
                    anim = "left";
                    sprite.scaleX = -1;
                } else if (mobAction.x > 0) {
                    anim = "right";
                    sprite.scaleX = 1;
                } else if (mobAction.y !== 0) {
                    anim = "updown";
                }

                let animKey = kMob.image + "_" + anim;
                let curAnimKey = sprite.anims.currentAnim ? sprite.anims.currentAnim.key : null;
                if (curAnimKey !== animKey || !sprite.anims.isPlaying) {
                    sprite.play(animKey);
                }
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
                this.bombSprites[bid].x = bx;
                this.bombSprites[bid].y = by;
            } else {
                // spawn new sprite for this bomb.
                // Use Sprite instead of Image so we can animate.
                let bomb = this.add.sprite(bx, by, "bombs");
                bomb.setDepth(DEPTH.BOMB);
                if (this.bombGroup) {
                    this.bombGroup.add(bomb);
                }

                // Show less frames if bomb will explode quicker.
                let frames: number[] = [];
                let secsRemaining = Math.floor(bombs[i].remaining);
                if (secsRemaining > 4) {
                    secsRemaining = 4;
                }

                for (let n = 4 - secsRemaining; n < 4; n++) {
                    frames.push(n);
                }

                // Create a unique animation key for this bomb's countdown.
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
                bomb.setOrigin(0.5);
                this.bombSprites[bid] = bomb;
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
                // Emitter already exists; update its position.
                this.explosionEmitters[eid].setPosition(ex, ey);
            } else {
                // Spawn new particle emitter for explosion.
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

                // Emit a burst of particles.
                emitter.explode(3);

                this.explosionEmitters[eid] = emitter;
            }
        }

        // Clean up.
        this.knownPlayers.cleanUp((pid) => {
            if (!this.curPlayer || pid !== this.curPlayer.id) {
                // Also delete the sprite if one exists.
                this.destroyPlayerSprite(pid);
                this.destroyPlayerName(pid);
            }
        });

        this.knownMobs.cleanUp((mid) => {
            this.destroyMobSprite(mid);
        });

        this.knownBombs.cleanUp((bid) => {
            this.destroyBombSprite(bid);
            // Also clean up the bomb-specific animation.
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

        this.updateStatus();
    }

    clientSidePrediction(): void {
        if (!this.curPlayer || !this.tmpPlayer) {
            return;
        }

        /*

        Client-side prediction.

        curPlayer represents the Player object last received from the server.
        playerSprites[pid] is the current and up-to-date sprite object.

        Use interpolation from curPlayer coords to current frame by processing
        all actionList inputs.
        */

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

        this.playerSprites[pid].x = this.tmpPlayer.x;
        this.playerSprites[pid].y = this.tmpPlayer.y;

        // Play animation according to direction.
        this.setSprite(this.playerSprites[pid], this.tmpPlayer.action, this.tmpPlayer.image);
    }

    movePlayer(player: Player): void {
        let deltaTime = 1.0 / targetFPS;

        // Move player.
        let mx = this.world.toMapX(player.x);
        let my = this.world.toMapY(player.y);
        let targetX = this.world.toScreenX(mx);
        let targetY = this.world.toScreenY(my);
        if (this.world.getcell(mx, my) === 1) {
            // ERROR: we're inside a wall - give up and wait for the server to
            // reposition us.
            return;
        }

        // Prevent illegal moves.
        let tmpaction = {
            x: player.action.x,
            y: player.action.y,
            deltaTime: player.action.deltaTime,
            fire: false,
            id: 0
        };

        this.fixPositionAndTmpAction(player, tmpaction, mx, my, targetX, targetY);

        // Lock to gridlines.
        let tolerance = this.world.tilewidth * 0.3;
        if (tmpaction.x !== 0) {
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

    setSprite(sprite: Phaser.GameObjects.Sprite, action: ActionData, imageKey: string): void {
        if (action.x !== 0 || action.y !== 0) {
            let anim: string = "";

            if (action.x < 0) {
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
