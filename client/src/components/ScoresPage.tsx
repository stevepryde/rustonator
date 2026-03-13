import { useState } from "react";
import { useScores } from "../hooks/useScores";
import type { ScoreEntry } from "../hooks/useScores";
import { uiAssets } from "../assets/uiAssets";
import Footer from "./Footer";

type ScoresTab = "today" | "month" | "all_time";

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

interface ScoresPageProps {
  onBack: () => void;
}

export default function ScoresPage({ onBack }: ScoresPageProps) {
  const { scores, loading } = useScores();
  const [tab, setTab] = useState<ScoresTab>("today");

  const entries =
    tab === "today"
      ? (scores?.today ?? [])
      : tab === "month"
        ? (scores?.month ?? [])
        : (scores?.all_time ?? []);

  return (
    <div
      className="w-full min-h-full p-4 overflow-y-auto retro-dotpattern"
      style={{
        backgroundImage: `url(${uiAssets.screen1Small})`,
        backgroundSize: "cover",
      }}
    >
      <div className="mx-auto w-full max-w-[560px] min-h-[calc(100vh-2rem)] flex flex-col justify-center">
        <div className="green-border-box text-center text-white">
          <button
            type="button"
            onClick={onBack}
            className="btn-solid w-full mb-4 font-bold sm:w-auto sm:min-w-[140px]"
          >
            BACK
          </button>

          <h2 className="text-2xl font-bold text-retro-green mb-4">SCORES</h2>

          <div className="flex flex-wrap justify-center gap-2 mb-4">
            <button
              type="button"
              onClick={() => setTab("today")}
              className={`border border-retro-green-dark px-4 py-1.5 text-sm rounded transition-all ${
                tab === "today"
                  ? "bg-retro-green-dark text-black font-bold shadow-[0_0_8px_var(--color-retro-green-glow)] border-retro-green-glow"
                  : "bg-transparent text-gray-300 hover:text-white"
              }`}
            >
              Today
            </button>
            <button
              type="button"
              onClick={() => setTab("month")}
              className={`border border-retro-green-dark px-4 py-1.5 text-sm rounded transition-all ${
                tab === "month"
                  ? "bg-retro-green-dark text-black font-bold shadow-[0_0_8px_var(--color-retro-green-glow)] border-retro-green-glow"
                  : "bg-transparent text-gray-300 hover:text-white"
              }`}
            >
              This Month
            </button>
            <button
              type="button"
              onClick={() => setTab("all_time")}
              className={`border border-retro-green-dark px-4 py-1.5 text-sm rounded transition-all ${
                tab === "all_time"
                  ? "bg-retro-green-dark text-black font-bold shadow-[0_0_8px_var(--color-retro-green-glow)] border-retro-green-glow"
                  : "bg-transparent text-gray-300 hover:text-white"
              }`}
            >
              All Time
            </button>
          </div>

          {loading && !scores ? (
            <p className="text-sm text-gray-300 py-6">Loading scores...</p>
          ) : (
            <ScoreTable entries={entries} />
          )}
        </div>

        <Footer />
      </div>
    </div>
  );
}
