import { DetonatorGame, targetFPS } from "./game";
import { gameBridge } from "../../bridge/GameBridge";

export class DetonatorGameOnline extends DetonatorGame {
  waitTimer: number = -1;

  serverAddress: string = "";
  serverSSL: boolean = false;
  joined: boolean = false;

  constructor(playerName: string, character: string, parentContainer: HTMLDivElement) {
    super(playerName, character, parentContainer);

    gameBridge.emit("loadingMessage", { message: "Connecting to server..." });
    gameBridge.emit("screenChange", { screen: "loading" });

    this.requestServer();
  }

  requestServer(): void {
    if (this.waitTimer >= 0) {
      window.clearTimeout(this.waitTimer);
    }

    this.serverAddress = import.meta.env.VITE_GAME_SERVER || location.host;
    this.serverSSL = import.meta.env.VITE_GAME_SERVER_SSL
      ? !!import.meta.env.VITE_GAME_SERVER_SSL
      : location.protocol === "https:";
    if (this.serverAddress) {
      window.clearTimeout(this.waitTimer);
      this.startGame();
    }
  }

  create() {
    super.create();

    // Init socket.
    const wsPath = import.meta.env.DEV ? "" : "/ws";
    if (this.serverSSL) {
      this.socket = new WebSocket("wss://" + this.serverAddress + wsPath, []);
    } else {
      this.socket = new WebSocket("ws://" + this.serverAddress + wsPath, []);
    }

    this.joined = false;
    this.setupSocket(this.socket);
    this.sendPing();
  }

  setupSocket(socket: WebSocket): void {
    socket.onopen = () => {
      let msg = JSON.stringify({
        "data": {
          "code": "JOINGAME",
          "data": {
            "name": this.playerName,
            "character": this.character
          }
        }
      });
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
        return;
      }
    };
  }

  handleMessage(code: string, data: any) {
    switch (code) {
      case "SPAWNPLAYER": {
        let [playerData, worldData] = data;
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
        data.players.push(data.player);
        this.updateVisiblePlayers(data.players, data.bombs, data.explosions, data.world, data.mobs);
        break;
      }
      case "DEAD": {
        this.playerDied(data);
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
      this.pingSent = false;
    }
  }

  sendPing(): void {
    if (!this.pingSent && this.socket && this.socket.readyState === 1) {
      let curMS = this.time.now;
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

  update(time: number, delta: number) {
    // Ping every 2 seconds.
    if (this.lagCounter++ > targetFPS * 2) {
      this.lagCounter = 0;
      if (!this.pingSent) {
        this.sendPing();
      }
      this.lagCounter = 0;
    }

    super.update(time, delta);
  }
}
