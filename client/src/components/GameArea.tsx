import type { PlayerData } from "../js/lib/common/player";
import Leaderboard from "./Leaderboard";
import PlayerStats from "./PlayerStats";
import OrientationWarning from "./OrientationWarning";

interface GameAreaProps {
  leaderboardPlayers: PlayerData[];
  currentPlayerId: string | null;
  statsPlayer: PlayerData | null;
  totalPlayers: number;
}

export default function GameArea({
  leaderboardPlayers,
  currentPlayerId,
  statsPlayer,
  totalPlayers,
}: GameAreaProps) {
  return (
    <>
      {/* Icon area - desktop only */}
      <div className="absolute right-[130px] top-[30px] p-2.5 hidden md:block">
        <img src="/assets/dsicon200.png" alt="game icon" />
      </div>

      <Leaderboard players={leaderboardPlayers} currentPlayerId={currentPlayerId} />
      <PlayerStats player={statsPlayer} totalPlayers={totalPlayers} />
      <OrientationWarning />
    </>
  );
}
