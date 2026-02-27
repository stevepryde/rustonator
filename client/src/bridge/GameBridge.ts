import { PlayerData } from "../js/lib/common/player";

export interface StartGameEvent {
  playerName: string;
  character: string;
}

export interface ScreenChangeEvent {
  screen: "menu" | "loading" | "game" | "postGame";
}

export interface LoadingMessageEvent {
  message: string;
}

export interface PlayerDiedEvent {
  reason: string;
  finalScore: number;
}

export interface LeaderboardUpdateEvent {
  players: PlayerData[];
  currentPlayerId: string | null;
}

export interface PlayerStatsUpdateEvent {
  player: PlayerData | null;
  totalPlayers: number;
}

export interface CanvasReadyEvent {
  container: HTMLDivElement;
}

interface BridgeEventMap {
  // React -> Phaser
  startGame: StartGameEvent;
  respawn: StartGameEvent;
  // Phaser -> React
  screenChange: ScreenChangeEvent;
  loadingMessage: LoadingMessageEvent;
  playerDied: PlayerDiedEvent;
  leaderboardUpdate: LeaderboardUpdateEvent;
  playerStatsUpdate: PlayerStatsUpdateEvent;
  canvasReady: CanvasReadyEvent;
}

type Listener<T> = (data: T) => void;

class GameBridge {
  private listeners: { [K in keyof BridgeEventMap]?: Listener<BridgeEventMap[K]>[] } = {};

  on<K extends keyof BridgeEventMap>(event: K, listener: Listener<BridgeEventMap[K]>): () => void {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }
    this.listeners[event]!.push(listener);

    return () => {
      const arr = this.listeners[event];
      if (arr) {
        const idx = arr.indexOf(listener);
        if (idx >= 0) arr.splice(idx, 1);
      }
    };
  }

  emit<K extends keyof BridgeEventMap>(event: K, data: BridgeEventMap[K]): void {
    const arr = this.listeners[event];
    if (arr) {
      for (const fn of arr) {
        fn(data);
      }
    }
  }
}

export const gameBridge = new GameBridge();
