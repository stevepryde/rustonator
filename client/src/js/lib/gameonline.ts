import { DetonatorGame, targetFPS, DeadData, PowerupData } from "./game";
import { GameState } from "./game";
import { WebUIManager, setElementDisplay } from "./web";
import { StateMachine } from "./statemachine";
import { PlayerData } from "./common/player";
import { ExplosionData } from "./common/explosion";
import { BombData } from "./common/bomb";
import { WorldData, ChunkData } from "./common/world";
import { MobData } from "./common/mob";

interface ServerData {
  ip: string;
  ssl?: boolean;
}

interface SocketData {
  code: string;
  data: any;
}

export class DetonatorGameOnline extends DetonatorGame {
  waitTimer: number = -1;

  serverAddress: string = "";
  serverSSL: boolean = false;
  joined: boolean = false;

  constructor(uiManager: WebUIManager, playerName: string, stateMachine: StateMachine) {
    super(uiManager, playerName, stateMachine);

    uiManager.showLoadingScreen("Connecting to server...");

    this.stateMachine.addTransition(GameState.InitGame, GameState.FindServer, () => {
      this.requestServer();
    });
    this.stateMachine.addTransition(GameState.FindServer, GameState.MainGame, () => {
      this.startGame();
    });

    this.stateMachine.setState(GameState.FindServer);
  }

  requestServer(): void {
    if (this.waitTimer >= 0) {
      window.clearTimeout(this.waitTimer);
    }

    // TODO: Set via .env?
    this.serverAddress = "127.0.0.1:9002";
    if (this.serverAddress) {
      this.serverSSL = false;
      window.clearTimeout(this.waitTimer);
      this.stateMachine.setState(GameState.MainGame);
    }
  }

  create() {
    if (!this.game) {
      console.error("Game is null");
      return;
    }

    if (this.game.device.desktop) {
      setElementDisplay({
        leaderboard: "block"
      });
    }
    super.create();

    // Init socket.
    if (this.serverSSL) {
      this.socket = new WebSocket("wss://" + this.serverAddress, []);
    } else {
      this.socket = new WebSocket("ws://" + this.serverAddress, []);
    }

    this.joined = false;
    this.setupSocket(this.socket);
    this.sendPing();

  }

  setupSocket(socket: WebSocket): void {
    socket.onopen = (event) => {
      console.log("Connected");
      let msg = JSON.stringify({"data": {"code": "JOINGAME", "data": this.playerName}});
      if (this.socket) {
        this.socket.send(msg);
      }
    };
    socket.onclose = () => {
      this.quitGame();
    };
    socket.onmessage = (event) => {
      try {
        let payload = JSON.parse(event.data).data;
        this.handleMessage(payload.code, payload.data);
      } catch (err) {
        console.log("Parse error: " + err + "\n" + event.data);
        return;
      }
    };
  }

  handleMessage(code: string, data: any) {
    switch (code) {
      case "SPAWNPLAYER": {
        let [playerData, worldData] = data;
        console.log(playerData);
        this.spawnPlayer(playerData);
        this.createWorld(worldData);
        this.joined = true;
        break;
      }
      case "POWERUP": {
        this.emitPowerup(data);
        break;
      }
      case "FRAMEDATA": {
        // Player is received separately, but just stick them in with players.
        data.players.push(data.player);
        this.updateVisiblePlayers(data.players, data.bombs, data.explosions, data.world, data.mobs);

        // DEBUG: simulate lag.
        // setTimeout(() => {
        //   this.updateVisiblePlayers(data.players, data.bombs, data.explosions, data.world, data.mobs);
        // }, 200);

        break;
      }
      case "DEAD": {
        this.playerDied(data);
        setTimeout(() => {
          this.socket?.close();
        }, 2000);
        break;
      }
      case "PONG": {
        this.updateLag(data);
        break;
      }
      default: {}
    }
  }

  updateLag(data: string): void {
    if (this.pingSent) {
      // let obj = JSON.parse(data);
      // let now = this?.game?.time.now;
      // if (now) {
      //   console.log(`LAG: ${now - obj.curMS}`);
      // }
      this.pingSent = false;
    }
  }


  sendPing(): void {
    if (!this.pingSent && this.game && this.socket && this.socket.readyState === 1) {
      let curMS = this.game.time.now;
      this.socket_wrapper("PING", JSON.stringify({ curMS }));
      this.pingSent = true;
    }
  }

  socket_wrapper(cmd: string, data: any): void {
    if (!this.socket || this.socket.readyState !== 1) {
      return;
    }
    if (!this.joined) {
      return;
    }

    let msg = JSON.stringify({"data": {"code": cmd, "data": data}});
    this.socket.send(msg);
  }

  update() {
    // Ping every 2 seconds.
    if (this.lagCounter++ > targetFPS * 2) {

      this.lagCounter = 0;
      if (!this.pingSent) {
        this.sendPing();
      }
      this.lagCounter = 0;
    }

    super.update();
  }
}
