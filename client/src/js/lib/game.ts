import "pixi";
import "p2";

import Phaser from "phaser-ce";
import {IMG_PREFIX, setElementDisplay, WebUIManager, withElement} from "./web";
import GameConfig from "./common/config";
import {Player, PlayerData} from "./common/player";
import {Action, ActionData} from "./common/action";
import {ChunkData, World, WorldData} from "./common/world";
import {ObjectPool} from "./objectpool";
import {Mob, MobData} from "./common/mob";
import {BombData} from "./common/bomb";
import {ExplosionData} from "./common/explosion";
import {StateMachine} from "./statemachine";
import {EffectType} from "./common/effect";

export const targetFPS = 30;
const GAME_DEBUG = false;

let DEBUG_LOG: any = {};
let FPS_COUNT: number = 0;
let FPS_INPUT_COUNT: number = 0;
if (GAME_DEBUG) {
    setInterval(() => {
        console.log("DEBUG: " + JSON.stringify(DEBUG_LOG));
        console.log("FPS: " + FPS_COUNT);
        console.log("FPS(INPUT): " + FPS_INPUT_COUNT);

        FPS_COUNT = 0;
        FPS_INPUT_COUNT = 0;
    }, 1000);
}

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

interface KeyBindingProperty {
    isDown(): boolean;
}

interface KeyBindings {
    up: KeyBindingProperty;
    down: KeyBindingProperty;
    left: KeyBindingProperty;
    right: KeyBindingProperty;
    fire: KeyBindingProperty;
    special: KeyBindingProperty;
}

export interface DeadData {
    reason: string;
}

export interface PowerupData {
    text: string;
}

//  The Google WebFont Loader will look for this object, so create it before loading the script.
let WebFontConfig = {
    //  "active" means all requested fonts have finished loading
    //  We set a 1 second delay before calling "createText".
    //  For some reason if we don't the browser cannot render the text the first time it's created.
    // active: function() {
    //   if (game) {
    //     game.time.events.add(Phaser.Timer.SECOND, createText, this);
    //   }
    // },

    //  The Google Fonts we want to load (specify as many as you like in the array)
    google: {
        families: ["Raleway"]
    }
};

export class DetonatorGame {
    uiManager: WebUIManager;
    stateMachine: StateMachine;
    playerName: string;
    gameState: GameState = GameState.Menu;
    game: Phaser.Game | null = null;
    canvasInfo: CanvasInfo;

    socket: WebSocket | null = null;

    lagCounter: number = 0;
    pingSent: boolean = false;

    world: World;
    worldSprites: (Phaser.Image | null)[] = [];
    lastTX: number = 0;
    lastTY: number = 0;

    cameraset: boolean = false;
    totalPlayers: number = 0;
    curPlayer: Player | null = null;
    curAction: ActionData | null = null;
    isDead: boolean = false;
    deadCounter: number = targetFPS * 3; // wait for 3 seconds before exiting game.
    quitFlag: boolean = false;
    deadSprite: Phaser.Sprite | null = null;
    actionList: ActionData[] = [];
    nextActionID: number = 0;

    lastClientMS: number = 0;
    curClientMS: number = 0;
    clientElapsedMS: number = 0;
    minMS: number = 1000 / targetFPS;
    deadReason: string = "";
    touchActions: TouchActions;

    mykeys: KeyBindings | null = null; // Main keys.
    altkeys: KeyBindings | null = null; // Alternative keys.
    fireflag: boolean = false;

    showGhost = false;
    tmpPlayer: Player | null = null;

    worldGroup: Phaser.Group | null = null;
    playerGroup: Phaser.Group | null = null;
    curPlayerGroup: Phaser.Group | null = null;
    mobGroup: Phaser.Group | null = null;
    bombGroup: Phaser.Group | null = null;
    explosionGroup: Phaser.Group | null = null;
    powerupGroup: Phaser.Group | null = null;
    shadeGroup: Phaser.Group | null = null;
    controlsGroup: Phaser.Group | null = null;
    controlSprites: { [x: string]: Phaser.Sprite } = {};
    playerSprites: { [x: string]: Phaser.Sprite } = {};
    mobSprites: { [x: string]: Phaser.Sprite } = {};
    bombSprites: { [x: string]: Phaser.Image } = {};
    explosionEmitters: { [x: string]: Phaser.Particles.Arcade.Emitter } = {};
    powerupSprites: Phaser.Text[] = [];
    playerNames: { [x: string]: Phaser.Text } = {};
    playerSpriteServer: Phaser.Sprite | null = null;
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
    leaderboardNames: Phaser.Text[] = [];
    leaderboardScores: Phaser.Text[] = [];
    leaderboardShade: Phaser.Image | null = null;
    scoreShade: Phaser.Image | null = null;
    scoreText: Phaser.Text | null = null;

    constructor(uiManager: WebUIManager, playerName: string, stateMachine: StateMachine) {
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

        this.game = new Phaser.Game(
            this.canvasInfo.width,
            this.canvasInfo.height,
            this.canvasInfo.canvasType,
            "gameCanvas",
            {
                preload: () => {
                    this.preload();
                },
                create: () => {
                    this.create();
                },
                update: () => {
                    this.update();
                },
                enableDebug: false
            }
        );
    }

    getCanvasInfo(): CanvasInfo {
        let gameConfig = new GameConfig();
        let screenX = gameConfig.screenX;
        let screenY = gameConfig.screenY;

        // Phaser.CANVAS is faster on firefox, AUTO is faster on chrome.
        // It seems that Chrome is the only browser that actually works well with
        // webGL - so use canvas for all others.
        let canvasType = Phaser.CANVAS;

        if (this.uiManager.isChrome()) {
            canvasType = Phaser.AUTO;
        }

        // Browser detection from here:
        // http://stackoverflow.com/questions/9847580/how-to-detect-safari-chrome-ie-firefox-and-opera-browser
        if (this.uiManager.isMobile()) {
            // adjust screen size to fit aspect ratio.
            let aspect = window.screen.width / window.screen.height;

            if (aspect < 1) {
                // phone is in portrait mode.
                aspect = window.screen.height / window.screen.width;
            }

            if (aspect > 1.33) {
                // reduce screenY to fit actual screen aspect.
                screenX = gameConfig.screenX;
                screenY = screenX * (1.0 / aspect);
            } else {
                // reduce screenX to fit actual screen aspect.
                screenY = gameConfig.screenY;
                screenX = screenY * aspect;
            }
        }

        // Remove all child elements of gameCanvas - phaser bug!
        withElement("gameCanvas", (canvas) => {
            while (canvas.hasChildNodes()) {
                let lastChild = canvas.lastChild;
                if (lastChild) {
                    canvas.removeChild(lastChild);
                }
            }
        });

        return {
            width: screenX,
            height: screenY,
            canvasType: canvasType
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

        if (this.game) {
            this.game.destroy();
        }
    }

    preload(): void {
        if (!this.game) {
            return;
        }

        let prefix = IMG_PREFIX;

        this.game.load.spritesheet("p1", prefix + "p1.png", 32, 32);
        this.game.load.spritesheet("p2", prefix + "p2.png", 32, 32);
        this.game.load.spritesheet("p3", prefix + "p3.png", 32, 32);
        this.game.load.spritesheet("p4", prefix + "p4.png", 32, 32);
        this.game.load.spritesheet("mob1", prefix + "mob1.png", 32, 32);
        this.game.load.spritesheet("tiles", prefix + "tileset1.png", 32, 32);
        this.game.load.spritesheet("explode", prefix + "explode.png", 32, 32);
        this.game.load.spritesheet("bombs", prefix + "bombtiles.png", 32, 32);
        this.game.load.spritesheet("controls", prefix + "controls.png", 32, 32);
        this.game.load.image("shade", prefix + "shade.png");

        this.game.load.script(
            "webfont",
            "https://ajax.googleapis.com/ajax/libs/webfont/1.4.7/webfont.js"
        );
    }

    create(): void {
        if (!this.game) {
            console.error("Game is null");
            return;
        }

        this.game.time.advancedTiming = true;

        // Phaser tweaks.
        this.mykeys = this.game.input.keyboard.addKeys({
            up: Phaser.KeyCode.UP,
            down: Phaser.KeyCode.DOWN,
            left: Phaser.KeyCode.LEFT,
            right: Phaser.KeyCode.RIGHT,
            fire: Phaser.KeyCode.SPACEBAR,
            special: Phaser.KeyCode.COMMA
        });
        this.altkeys = this.game.input.keyboard.addKeys({
            up: Phaser.KeyCode.W,
            down: Phaser.KeyCode.S,
            left: Phaser.KeyCode.A,
            right: Phaser.KeyCode.D,
            fire: Phaser.KeyCode.CONTROL,
            special: Phaser.KeyCode.ALT
        });

        this.curAction = new Action().toJSON();

        this.worldGroup = this.game.add.group();
        this.worldGroup.z = -100;

        this.bombGroup = this.game.add.group();
        this.bombGroup.z = 10;

        this.playerGroup = this.game.add.group();
        this.playerGroup.z = 20;

        this.curPlayerGroup = this.game.add.group();
        this.curPlayerGroup.z = 30;

        this.mobGroup = this.game.add.group();
        this.mobGroup.z = 35;

        this.explosionGroup = this.game.add.group();
        this.explosionGroup.z = 40;

        this.powerupGroup = this.game.add.group();
        this.powerupGroup.z = 50;

        this.shadeGroup = this.game.add.group();
        this.shadeGroup.z = 90;

        this.controlsGroup = this.game.add.group();
        this.controlsGroup.z = 100;

        // Init display.
        this.game.scale.scaleMode = Phaser.ScaleManager.SHOW_ALL;
        this.game.scale.fullScreenScaleMode = Phaser.ScaleManager.SHOW_ALL;
        this.game.scale.windowConstraints.bottom = "visual";
        this.game.stage.smoothed = false;
        this.game.stage.backgroundColor = "#000000";
        this.game.stage.disableVisibilityChange = true;

        if (this.game.device.desktop) {
            this.touchEnabled = false;
            this.iconDisplayed = true;

            setElementDisplay({
                iconarea: "block",
                player: "block"
            });

            this.game.scale.stopFullScreen();

            this.playerStatsDisplayed = true;
        } else {
            this.touchEnabled = true;
            this.game.scale.forceOrientation(true, false);
            this.game.scale.enterIncorrectOrientation.add(this.handleIncorrect);
            this.game.scale.leaveIncorrectOrientation.add(this.handleCorrect);

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

    handleIncorrect(): void {
        if (this.game && !this.game.device.desktop) {
            setElementDisplay({ orientwarning: "block" });
        }
    }

    handleCorrect(): void {
        if (this.game && !this.game.device.desktop) {
            setElementDisplay({ orientwarning: "none" });
        }
    }

    update(): void {
        if (this.quitFlag) {
            return;
        }
        if (!this.game) {
            console.error("Game is null!");
            return;
        }

        FPS_COUNT++;

        this.curClientMS = this.game.time.now;
        if (this.lastClientMS === 0) {
            this.lastClientMS = this.curClientMS - 1000 / targetFPS;
        }

        this.clientElapsedMS = this.curClientMS - this.lastClientMS;

        if (!this.cameraset && this.curPlayer && this.curPlayer.id in this.playerSprites) {
            this.game.camera.follow(this.playerSprites[this.curPlayer.id]);
            this.cameraset = true;
        }

        if (!this.isDead) {
            // client-side prediction.
            this.clientSidePrediction();
        }

        this.handleTouch();

        if (this.clientElapsedMS >= this.minMS) {


            FPS_INPUT_COUNT++;
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
                    } else {
                        this.deadSprite = this.game.add.sprite(
                            this.curPlayer.x,
                            this.curPlayer.y,
                            this.curPlayer.image,
                            1
                        );
                        this.deadSprite.anchor.set(0.5);
                        if (this.curPlayerGroup) {
                            this.curPlayerGroup.add(this.deadSprite);
                        }
                    }
                }
            }

            // Fly away...
            if (this.deadSprite) {
                this.deadSprite.y -= 0.5;
                this.deadSprite.alpha -= 0.01;
                if (this.deadSprite.alpha <= 0) {
                    this.deadSprite.alpha = 0;
                }
                this.deadSprite.scale.x *= 1.05;
                this.deadSprite.scale.y *= 1.05;
            }

            this.deadCounter--;
            if (this.deadCounter <= 0) {
                this.quitFlag = true;
            }
        }
    }

    handleTouch(): void {
        // because phaser event-based touch input is fucked.
        if (!this.game) {
            return;
        }

        this.touchActions = {
            up: false,
            down: false,
            left: false,
            right: false,
            bomb: false
        };
        this.checkPointer(this.game.input.pointer1);
        this.checkPointer(this.game.input.pointer2);
    }

    checkPointer(pointer: Phaser.Pointer): void {
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

    spriteContains(sprite: Phaser.Sprite, x: number, y: number): boolean {
        if (!this.game) {
            return false;
        }

        let sx = sprite.x - this.game.camera.x;
        let sy = sprite.y - this.game.camera.y;

        if (x >= sx && x < sx + sprite.width && y >= sy && y < sy + sprite.height) {
            return true;
        }

        return false;
    }

    goFull(): void {
        if (!this.game) {
            return;
        }

        if (this.touchEnabled) {
            if (!this.game.scale.isFullScreen) {
                this.game.scale.startFullScreen(false, true);
            } else {
                this.game.scale.stopFullScreen();
            }
        }
    }

    addTouchControls(): void {
        if (!this.game || !this.controlsGroup) {
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
        let sprite = this.game.add.sprite(controlsx + size + gap, controlsy, "controls", 0);
        this.controlSprites["up"] = sprite;
        sprite.fixedToCamera = true;
        sprite.scale.x = scale;
        sprite.scale.y = scale;
        sprite.alpha = 0.8;
        this.controlsGroup.add(sprite);

        // DOWN
        sprite = this.game.add.sprite(
            controlsx + size + gap,
            controlsy + size + gap + size + gap,
            "controls",
            1
        );
        sprite.fixedToCamera = true;
        this.controlSprites["down"] = sprite;
        sprite.inputEnabled = true;
        sprite.scale.x = scale;
        sprite.scale.y = scale;
        sprite.alpha = 0.8;
        this.controlsGroup.add(sprite);

        // LEFT
        sprite = this.game.add.sprite(controlsx, controlsy + size + gap, "controls", 2);
        sprite.fixedToCamera = true;
        this.controlSprites["left"] = sprite;
        sprite.inputEnabled = true;
        sprite.scale.x = scale;
        sprite.scale.y = scale;
        sprite.alpha = 0.8;
        this.controlsGroup.add(sprite);

        // RIGHT
        sprite = this.game.add.sprite(
            controlsx + size + gap + size + gap,
            controlsy + size + gap,
            "controls",
            3
        );
        sprite.fixedToCamera = true;
        this.controlSprites["right"] = sprite;
        sprite.inputEnabled = true;
        sprite.scale.x = scale;
        sprite.scale.y = scale;
        sprite.alpha = 0.8;
        this.controlsGroup.add(sprite);

        // BOMB
        sprite = this.game.add.sprite(right - (10 + size), bottom - (10 + size), "controls", 4);
        sprite.fixedToCamera = true;
        this.controlSprites["bomb"] = sprite;
        sprite.inputEnabled = true;
        sprite.scale.x = scale;
        sprite.scale.y = scale;
        sprite.alpha = 0.8;
        this.controlsGroup.add(sprite);

        // Don't show fullscreen button on Android - point users to app instead.
        // Don't show the fullscreen button on iPhone/iPad either - it doesn't work.
        if (!this.uiManager.isAndroid() && !this.uiManager.isApple()) {
            // FULLSCREEN TOGGLE
            sprite = this.game.add.sprite(10, 10, "controls", 5);
            sprite.fixedToCamera = true;
            this.controlSprites["fs"] = sprite;
            sprite.events.onInputDown.add(this.goFull);
            this.controlsGroup.add(sprite);
            sprite.scale.x = 2;
            sprite.scale.y = 2;
            sprite.alpha = 0.8;
            sprite.inputEnabled = true;
        } else if (this.uiManager.isApple()) {
            // EXIT BUTTON
            sprite = this.game.add.sprite(10, 10, "controls", 6);
            sprite.fixedToCamera = true;
            this.controlSprites["exit"] = sprite;
            sprite.events.onInputDown.add(this.uiManager.showExitPopup);
            this.controlsGroup.add(sprite);
            sprite.scale.x = 2;
            sprite.scale.y = 2;
            sprite.alpha = 0.8;
            sprite.inputEnabled = true;
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
        // We died. Too bad so sad.
        this.isDead = true;
        this.deadReason = data;
    }

    updateStatus(): void {
        if (!this.game) {
            return;
        }

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

            // status += "   FPS: " + this.game.time.fps;

            if (!this.scoreText) {
                let text = this.game.add.text(310, this.canvasInfo.height - 20, status);
                text.fixedToCamera = true;
                text.anchor.setTo(0);
                text.font = "Raleway";
                text.fontSize = 12;
                text.fill = "#ffffff";
                text.alpha = 0.8;
                text.align = "left";
                text.strokeThickness = 0;
                text.setShadow(1, 1, "rgba(0, 0, 0, 0.5)", 0);
                if (this.controlsGroup) {
                    this.controlsGroup.add(text);
                }
                this.scoreText = text;
            } else {
                this.scoreText.text = status;
            }

            if (!this.scoreShade) {
                this.scoreShade = this.game.add.image(300, this.canvasInfo.height - 25, "shade");
                this.scoreShade.fixedToCamera = true;
                this.scoreShade.anchor.setTo(0);
                if (this.shadeGroup) {
                    this.shadeGroup.add(this.scoreShade);
                }
            }

            this.scoreShade.width = this.scoreText.width + 20;
            this.scoreShade.height = 20;

            return;
        }

        // Only display if the window is big enough.
        // let w = Math.max(document.documentElement.clientWidth, window.innerWidth || 0);
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

            // status += "<br /><br />FPS: " + game.time.fps;
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
            this.playerSprites[pid].kill();
            this.playerSprites[pid].destroy();
            delete this.playerSprites[pid];
        }
    }

    destroyMobSprite(mid: string): void {
        if (mid in this.mobSprites) {
            this.mobSprites[mid].kill();
            this.mobSprites[mid].destroy();
            delete this.mobSprites[mid];
        }
    }

    destroyPlayerName(pid: string): void {
        if (pid in this.playerNames) {
            this.playerNames[pid].kill();
            this.playerNames[pid].destroy();
            delete this.playerNames[pid];
        }
    }

    destroyBombSprite(bid: string): void {
        if (bid in this.bombSprites) {
            this.bombSprites[bid].kill();
            this.bombSprites[bid].destroy();
            delete this.bombSprites[bid];
        }
    }

    destroyExplosion(eid: string): void {
        if (eid in this.explosionEmitters) {
            this.explosionEmitters[eid].removeAll(true);
            if (this.explosionGroup) {
                this.explosionGroup.remove(this.explosionEmitters[eid], true);
            }
            delete this.explosionEmitters[eid];
        }
    }

    createWorld(data: WorldData): void {
        if (!this.game) {
            console.error("GAME NOT RUNNING!");
            return;
        }

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

        this.game.world.setBounds(
            0,
            0,
            this.world.width * this.world.tilewidth,
            this.world.height * this.world.tileheight
        );
    }

    updateWorld(data: ChunkData): void {
        if (!this.game) {
            return;
        }

        let tx = data.tx;
        let ty = data.ty;
        let chunkwidth = data.width;
        let chunkheight = data.height;
        let mapdata = data.data;
        let index;
        let realIndex;
        let val;
        let mx, my;
        let tile;

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
                        sprite.kill();

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
                        sprite.kill();

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
                    if (this.worldGroup) {
                        tile = this.worldGroup.getFirstExists(
                            false, // not exists.
                            false, // don't create if null.
                            mx * this.world.tilewidth,
                            my * this.world.tileheight,
                            "tiles",
                            val
                        );
                    }

                    if (tile == null) {
                        tile = this.game.add.image(
                            mx * this.world.tilewidth,
                            my * this.world.tileheight,
                            "tiles",
                            val
                        );
                        tile.anchor.set(0, 0);
                        if (this.worldGroup) {
                            this.worldGroup.add(tile);
                        }
                    } else {
                        tile.revive();
                    }

                    this.worldSprites[realIndex] = tile;
                } else {
                    tile.frame = val;
                }

                realIndex++;
            }
        }


        // DEBUG: check number of valid tiles.
        // let count = 0;
        // for (let i = 0; i < worldSprites.length; i++) {
        //     if (worldSprites[i] != null) {
        //         count++;
        //     }
        // }
    }

    emitPowerup(data: string): void {
        if (!this.curPlayer || !this.game) {
            return;
        }

        let text = this.game.add.text(this.curPlayer.x, this.curPlayer.y, data);
        text.anchor.setTo(0.5);
        text.font = "Raleway";
        text.fontSize = 12;
        let first = data.charAt(0);
        if (first === "+") {
            text.fill = "#00ff00";
        } else if (first === "-") {
            text.fill = "#ff0000";
        } else {
            text.fill = "#ffffff";
        }
        text.alpha = 1.0;

        text.align = "center";
        // text.stroke = "#000000";
        text.strokeThickness = 0;
        text.setShadow(1, 1, "rgba(0,0,0,0.8)", 0);

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
        if (!this.game) {
            return;
        }

        let pid: string;
        let bid: string;
        let eid: string;
        let mid: string;
        let i;
        let sprite;

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

        DEBUG_LOG["curPlayer"] = this.curPlayer;

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

                    this.setSprite(this.playerSprites[pid], kPlayer.action);

                    this.movePlayerName(kPlayer);
                }

                // Invincibility?
                if (kPlayer.hasFlag(2) && this.flickerToggle) {
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
                        this.playerSpriteServer = this.game.add.sprite(kPlayer.x, kPlayer.y, kPlayer.image);
                        this.playerSpriteServer.anchor.set(0.5);

                        this.playerSpriteServer.animations.add("down", [0, 1, 2, 1]);
                        this.playerSpriteServer.animations.add("up", [3, 4, 5, 4]);

                        // Even though left and right are the same, we need a different
                        // label to differentiate between them.
                        this.playerSpriteServer.animations.add("left", [6, 7, 9, 7, 6, 7, 8, 7]);
                        this.playerSpriteServer.animations.add("right", [6, 7, 9, 7, 6, 7, 8, 7]);

                        if (this.curPlayerGroup) {
                            this.curPlayerGroup.add(this.playerSpriteServer);
                        }
                    }

                    sprite = this.game.add.sprite(kPlayer.x, kPlayer.y, kPlayer.image);
                    if (this.curPlayerGroup) {
                        this.curPlayerGroup.add(sprite);
                    }
                } else {
                    sprite = this.game.add.sprite(kPlayer.x, kPlayer.y, kPlayer.image);
                    if (this.playerGroup) {
                        this.playerGroup.add(sprite);
                    }

                    this.makePlayerName(kPlayer);
                }

                sprite.anchor.set(0.5);
                sprite.animations.add("down", [0, 1, 2, 1]);
                sprite.animations.add("up", [3, 4, 5, 4]);

                // Even though left and right are the same, we need a different
                // label to differentiate between them.
                sprite.animations.add("left", [6, 7, 9, 7, 6, 7, 8, 7]);
                sprite.animations.add("right", [6, 7, 9, 7, 6, 7, 8, 7]);

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

                // sprite...
                let mobAction = mobs[i].action;
                let anim = "updown";

                if (mobAction.x < 0) {
                    anim = "left";
                    sprite.scale.x = -1;
                } else if (mobAction.x > 0) {
                    anim = "right";
                    sprite.scale.x = 1;
                } else if (mobAction.y !== 0) {
                    anim = "updown";
                }

                let curAnim = sprite.animations.currentAnim;
                if (!curAnim || curAnim.name != anim || !curAnim.isPlaying) {
                    sprite.animations.play(anim, 10, true);
                }
            } else {
                // spawn new sprite for this mob.
                if (!kMob.image) {
                    // default image.
                    kMob.image = "mob1";
                }

                sprite = this.game.add.sprite(kMob.x, kMob.y, kMob.image);
                if (this.mobGroup) {
                    this.mobGroup.add(sprite);
                }

                sprite.anchor.set(0.5);
                sprite.animations.add("updown", [0, 1]);

                // Even though left and right are the same, we need a different
                // label to differentiate between them.
                sprite.animations.add("left", [2, 3]);
                sprite.animations.add("right", [2, 3]);

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
                let bomb = this.game.add.image(bx, by, "bombs");
                if (this.bombGroup) {
                    this.bombGroup.add(bomb);
                }

                // Show less frames if bomb will explode quicker.
                let frames = [];
                let secsRemaining = Math.floor(bombs[i].remaining);
                if (secsRemaining > 4) {
                    secsRemaining = 4;
                }

                for (let n = 4 - secsRemaining; n < 4; n++) {
                    frames.push(n);
                }

                bomb.animations.add("blow", frames);
                bomb.animations.play("blow", 1, false);
                bomb.anchor.set(0.5);
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
                this.explosionEmitters[eid].emitX = ex;
                this.explosionEmitters[eid].emitY = ey;
            } else {
                // spawn new emitter for explosion.
                let emitter = this.game.add.emitter(ex, ey, 3); // max particles.
                if (this.explosionGroup) {
                    this.explosionGroup.add(emitter);
                }

                emitter.makeParticles("explode", [0, 1, 2, 3, 4, 5]);
                emitter.gravity = new Phaser.Point(0, 0);
                // Make these go a little bit longer.
                let ms = Math.floor(explosions[i].remaining * 1200);
                emitter.setAlpha(1, 0, ms);
                emitter.setScale(1.0, 0.5, 1.0, 0.5, ms);
                emitter.minParticleSpeed.setTo(-32, -32);
                emitter.maxParticleSpeed.setTo(32, 32);
                emitter.setRotation(0, 100);
                emitter.start(true, ms, 25, 3);

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
                this.setSprite(this.playerSpriteServer, this.curPlayer.action);
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
        DEBUG_LOG["actionList"] = this.actionList.length;

        this.playerSprites[pid].x = this.tmpPlayer.x;
        this.playerSprites[pid].y = this.tmpPlayer.y;

        // Play animation according to direction.
        this.setSprite(this.playerSprites[pid], this.tmpPlayer.action);
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

        player.updateWithTempAction(tmpaction, deltaTime);

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

    setSprite(sprite: Phaser.Sprite, action: ActionData): void {
        if (action.x !== 0 || action.y !== 0) {
            let anim: string = "";

            if (action.x < 0) {
                anim = "left";
                sprite.scale.x = -1;
            } else if (action.x > 0) {
                anim = "right";
                sprite.scale.x = 1;
            } else if (action.y < 0) {
                anim = "up";
            } else if (action.y > 0) {
                anim = "down";
            }

            if (anim && anim.length > 0) {
                let curAnim = sprite.animations.currentAnim;
                if (!curAnim || curAnim.name !== anim || !curAnim.isPlaying) {
                    sprite.animations.play(anim, 10, true);
                }
            }
        } else {
            sprite.animations.stop();
        }
    }

    makePlayerName(player: Player): boolean {
        if (!this.game || player.id in this.playerNames) {
            return false;
        }

        if (!player.name) {
            return false;
        }

        let text = this.game.add.text(player.x, player.y - 20, player.name);
        text.anchor.setTo(0.5);
        text.font = "Raleway";
        text.fontSize = 12;
        text.fill = "#ffffff";
        text.alpha = 0.8;
        text.align = "center";
        text.strokeThickness = 0;
        text.setShadow(1, 1, "rgba(0,0,0,0.5)", 0);

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
