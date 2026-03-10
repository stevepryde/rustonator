import { buildTimedStatuses } from "../lib/playerStatus";
import type { PlayerData } from "../js/lib/common/player";

interface TimedStatusStripProps {
  player: PlayerData | null;
  maxVisible?: number;
}

const toneClasses = {
  good: "border-emerald-400/60 bg-emerald-500/15 text-emerald-100",
  bad: "border-amber-400/60 bg-amber-500/15 text-amber-50",
  hot: "border-orange-400/60 bg-orange-500/15 text-orange-50",
} as const;

export default function TimedStatusStrip({ player, maxVisible = 4 }: TimedStatusStripProps) {
  const statuses = buildTimedStatuses(player);
  if (statuses.length === 0) {
    return null;
  }

  const visible = statuses.slice(0, maxVisible);
  const overflow = statuses.length - visible.length;

  return (
    <div className="mt-2">
      <div className="mb-1 text-[10px] uppercase tracking-[0.22em] text-white/55">Active Effects</div>
      <div className="flex flex-wrap gap-1.5">
        {visible.map((status) => (
          <div
            key={status.key}
            className={`inline-flex items-center gap-1 rounded border px-2 py-1 text-xs font-bold ${toneClasses[status.tone]}`}
            title={status.label}
          >
            <span className="text-sm leading-none">{status.icon}</span>
            {status.detail ? <span>{status.detail}</span> : null}
            {status.remaining !== undefined ? (
              <span className="text-white/80">{Math.max(1, Math.ceil(status.remaining))}s</span>
            ) : null}
          </div>
        ))}
        {overflow > 0 ? (
          <div className="inline-flex items-center gap-1 rounded border border-white/20 bg-white/10 px-2 py-1 text-xs font-bold text-white/80">
            <span className="text-sm leading-none">➕</span>
            <span>{overflow}</span>
          </div>
        ) : null}
      </div>
    </div>
  );
}
