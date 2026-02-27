import { useCallback, useEffect, useRef, useState } from "react";
import { gameBridge, type PlayerDiedEvent, type LeaderboardUpdateEvent, type PlayerStatsUpdateEvent } from "../bridge/GameBridge";
import { DetonatorGameOnline } from "../js/lib/gameonline";
import type { DetonatorGame } from "../js/lib/game";
import type { PlayerData } from "../js/lib/common/player";

export type Screen = "menu" | "loading" | "game" | "postGame";

export interface GameState {
  screen: Screen;
  playerName: string;
  character: string;
  loadingMessage: string;
  deathReason: string;
  finalScore: number;
  leaderboardPlayers: PlayerData[];
  currentPlayerId: string | null;
  statsPlayer: PlayerData | null;
  totalPlayers: number;
}

function loadStored<T>(key: string, fallback: T): T {
  try {
    const v = localStorage.getItem(key);
    return v !== null ? (JSON.parse(v) as T) : fallback;
  } catch {
    return fallback;
  }
}

export function useGameState() {
  const [state, setState] = useState<GameState>({
    screen: "menu",
    playerName: loadStored("detonator_playerName", ""),
    character: loadStored("detonator_character", "p1"),
    loadingMessage: "Connecting to server...",
    deathReason: "",
    finalScore: 0,
    leaderboardPlayers: [],
    currentPlayerId: null,
    statsPlayer: null,
    totalPlayers: 0,
  });

  const gameContainerRef = useRef<HTMLDivElement>(null);
  const gameObjRef = useRef<DetonatorGame | null>(null);

  // Keep refs in sync so callbacks always read the latest values.
  const playerNameRef = useRef(state.playerName);
  const characterRef = useRef(state.character);
  playerNameRef.current = state.playerName;
  characterRef.current = state.character;

  useEffect(() => {
    const unsubs = [
      gameBridge.on("screenChange", ({ screen }) => {
        setState((s) => ({ ...s, screen }));
      }),
      gameBridge.on("loadingMessage", ({ message }) => {
        setState((s) => ({ ...s, loadingMessage: message }));
      }),
      gameBridge.on("playerDied", ({ reason, finalScore }: PlayerDiedEvent) => {
        setState((s) => ({ ...s, deathReason: reason, finalScore }));
      }),
      gameBridge.on("leaderboardUpdate", ({ players, currentPlayerId }: LeaderboardUpdateEvent) => {
        setState((s) => ({ ...s, leaderboardPlayers: players, currentPlayerId }));
      }),
      gameBridge.on("playerStatsUpdate", ({ player, totalPlayers }: PlayerStatsUpdateEvent) => {
        setState((s) => ({ ...s, statsPlayer: player, totalPlayers }));
      }),
    ];

    return () => unsubs.forEach((fn) => fn());
  }, []);

  const launchGame = useCallback(() => {
    if (gameObjRef.current) {
      gameObjRef.current.cleanup();
      gameObjRef.current = null;
    }

    const playerName = playerNameRef.current.replace(/(<([^>]+)>)/gi, "");
    const character = characterRef.current;

    const container = gameContainerRef.current;
    if (container) {
      gameObjRef.current = new DetonatorGameOnline(playerName, character, container);
    }
  }, []);

  const setPlayerName = useCallback((playerName: string) => {
    localStorage.setItem("detonator_playerName", JSON.stringify(playerName));
    setState((s) => ({ ...s, playerName }));
  }, []);

  const setCharacter = useCallback((character: string) => {
    localStorage.setItem("detonator_character", JSON.stringify(character));
    setState((s) => ({ ...s, character }));
  }, []);

  return {
    ...state,
    gameContainerRef,
    setPlayerName,
    setCharacter,
    startGame: launchGame,
    respawn: launchGame,
  };
}
