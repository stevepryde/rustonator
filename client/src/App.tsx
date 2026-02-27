import { useGameState } from "./hooks/useGameState";
import StartMenu from "./components/StartMenu";
import LoadingScreen from "./components/LoadingScreen";
import GameArea from "./components/GameArea";
import PostGameModal from "./components/PostGameModal";

export default function App() {
  const {
    screen,
    playerName,
    character,
    loadingMessage,
    deathReason,
    finalScore,
    leaderboardPlayers,
    currentPlayerId,
    statsPlayer,
    totalPlayers,
    gameContainerRef,
    setPlayerName,
    setCharacter,
    startGame,
    respawn,
  } = useGameState();

  return (
    <>
      {screen === "menu" && (
        <StartMenu
          playerName={playerName}
          character={character}
          onNameChange={setPlayerName}
          onCharacterChange={setCharacter}
          onStart={startGame}
        />
      )}
      {screen === "loading" && <LoadingScreen message={loadingMessage} />}
      {screen === "postGame" && (
        <PostGameModal
          reason={deathReason}
          score={finalScore}
          onRespawn={respawn}
        />
      )}

      {/* Phaser canvas container — always in DOM so Phaser can mount during loading */}
      <div
        className={screen === "game" ? "m-5 h-[90vh] p-1 rounded retro-dotpattern box-border relative" : "hidden"}
      >
        {screen === "game" && (
          <GameArea
            leaderboardPlayers={leaderboardPlayers}
            currentPlayerId={currentPlayerId}
            statsPlayer={statsPlayer}
            totalPlayers={totalPlayers}
          />
        )}
        <div ref={gameContainerRef} className="w-full h-full m-0 p-0" />
      </div>
    </>
  );
}
