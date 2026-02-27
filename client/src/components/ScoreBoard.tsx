import { useState } from "react";
import type { ScoreBuckets, ScoreEntry } from "../hooks/useScores";

function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function ScoreTable({ entries }: { entries: ScoreEntry[] }) {
  return (
    <table className="w-full text-center font-bold text-white">
      <tbody>
        {Array.from({ length: 10 }, (_, i) => {
          const entry = entries[i];
          return (
            <tr key={i}>
              <td className="w-[5%] text-left align-top p-1 text-sm">{i + 1}.</td>
              <td
                className="w-3/4 text-left align-top p-1 text-sm break-words"
                dangerouslySetInnerHTML={{
                  __html: entry ? escapeHtml(entry.name) : "---",
                }}
              />
              <td className="w-[20%] text-right align-top p-1 text-sm">
                {entry ? entry.score : "-"}
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}

interface ScoreBoardProps {
  scores: ScoreBuckets | null;
  variant: "today" | "top";
}

export default function ScoreBoard({ scores, variant }: ScoreBoardProps) {
  const [tab, setTab] = useState<"month" | "alltime">("month");

  if (variant === "today") {
    return (
      <div className="text-center text-white">
        <h3 className="text-xl font-bold mb-2 text-retro-green">TODAY</h3>
        <ScoreTable entries={scores?.today ?? []} />
      </div>
    );
  }

  return (
    <div className="text-center text-white">
      <div className="flex justify-center gap-1.5 mb-2">
        <button
          type="button"
          onClick={() => setTab("month")}
          className={`bg-transparent border border-retro-green-dark px-4 py-1.5 cursor-pointer text-sm rounded transition-colors ${
            tab === "month"
              ? "bg-retro-green-dark text-black font-bold"
              : "text-gray-400 hover:text-white"
          }`}
        >
          This Month
        </button>
        <button
          type="button"
          onClick={() => setTab("alltime")}
          className={`bg-transparent border border-retro-green-dark px-4 py-1.5 cursor-pointer text-sm rounded transition-colors ${
            tab === "alltime"
              ? "bg-retro-green-dark text-black font-bold"
              : "text-gray-400 hover:text-white"
          }`}
        >
          All Time
        </button>
      </div>
      <ScoreTable entries={tab === "month" ? (scores?.month ?? []) : (scores?.all_time ?? [])} />
    </div>
  );
}
