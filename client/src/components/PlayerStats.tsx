import type { PlayerData } from "../js/lib/common/player";
import TimedStatusStrip from "./TimedStatusStrip";

interface PlayerStatsProps {
  player: PlayerData | null;
  totalPlayers: number;
}

export default function PlayerStats({ player, totalPlayers }: PlayerStatsProps) {
  if (!player) return null;

  return (
    <div className="absolute right-[30px] top-[600px] w-[400px] p-2.5 bg-black/70 text-white font-bold text-left hidden md:block">
      <div className="text-retro-yellow text-center text-base font-bold">
        {player.name}
      </div>
      <hr className="border-white/30 my-1" />
      <table className="w-full">
        <tbody>
          <tr>
            <td className="w-1/2">
              <b>SCORE:</b> {player.score}
            </td>
            <td>
              <b>BOMBS:</b> {player.maxBombs}
            </td>
          </tr>
          <tr>
            <td>
              <b>RANGE:</b> {player.range}
            </td>
            <td>
              {player.rank && totalPlayers ? (
                <>
                  <b>RANK:</b> {player.rank} of {totalPlayers}
                </>
              ) : (
                "\u00A0"
              )}
            </td>
          </tr>
        </tbody>
      </table>
      <TimedStatusStrip player={player} />
    </div>
  );
}
