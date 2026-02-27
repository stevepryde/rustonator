import type { PlayerData } from "../js/lib/common/player";

function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

interface LeaderboardProps {
  players: PlayerData[];
  currentPlayerId: string | null;
}

export default function Leaderboard({ players, currentPlayerId }: LeaderboardProps) {
  const sorted = players
    .filter((p) => p.active && p.name)
    .sort((a, b) => b.score - a.score)
    .slice(0, 10);

  return (
    <div className="absolute right-[30px] top-[300px] w-[400px] p-2.5 bg-black/70 text-white font-bold text-left hidden md:block">
      <div className="text-retro-yellow text-center text-base mb-1">PLAYERS</div>
      <table className="w-full">
        <tbody>
          {sorted.map((p) => {
            const isMe = p.id === currentPlayerId;
            return (
              <tr key={p.id} style={{ color: isMe ? "#5ee65c" : "#fff" }}>
                <td
                  className="text-left text-sm px-1 py-px"
                  dangerouslySetInnerHTML={{ __html: escapeHtml(p.name) }}
                />
                <td className="text-right text-sm px-1 py-px">{p.score}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
