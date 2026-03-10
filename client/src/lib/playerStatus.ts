import { EffectType, type EffectData } from "../js/lib/common/effect";

export interface TimedStatusSource {
  effects: EffectData[];
  scoreMultiplier: number;
  scoreMultiplierRemaining: number;
  remoteBombCharges: number;
}

export interface TimedStatusItem {
  key: string;
  icon: string;
  label: string;
  remaining?: number;
  detail?: string;
  tone: "good" | "bad" | "hot";
}

function effectToStatus(effect: EffectData): TimedStatusItem | null {
  switch (effect.effectType) {
    case EffectType.SpeedUp:
      return { key: "speedup", icon: "⚡", label: "Speed Up", remaining: effect.remaining, tone: "good" };
    case EffectType.SlowDown:
      return { key: "slowdown", icon: "🐢", label: "Slow Down", remaining: effect.remaining, tone: "bad" };
    case EffectType.Invincibility:
      return { key: "invincibility", icon: "🛡️", label: "Invincible", remaining: effect.remaining, tone: "good" };
    case EffectType.InputInversion:
      return { key: "invert", icon: "🙃", label: "Input Inverted", remaining: effect.remaining, tone: "bad" };
    default:
      return null;
  }
}

export function buildTimedStatuses(player: TimedStatusSource | null): TimedStatusItem[] {
  if (!player) {
    return [];
  }

  const statuses: TimedStatusItem[] = [];

  if (player.scoreMultiplier > 1 && player.scoreMultiplierRemaining > 0) {
    statuses.push({
      key: "multiplier",
      icon: "🔥",
      label: "Kill Multiplier",
      remaining: player.scoreMultiplierRemaining,
      detail: `x${player.scoreMultiplier}`,
      tone: "hot",
    });
  }

  if (player.remoteBombCharges > 0) {
    statuses.push({
      key: "remote-bombs",
      icon: "😈",
      label: "Remote Bombs",
      detail: `x${player.remoteBombCharges}`,
      tone: "hot",
    });
  }

  for (const effect of player.effects) {
    const status = effectToStatus(effect);
    if (status && effect.remaining > 0) {
      statuses.push(status);
    }
  }

  return statuses.sort((a, b) => {
    if (a.key === "multiplier") {
      return -1;
    }
    if (b.key === "multiplier") {
      return 1;
    }
    if (a.key === "remote-bombs") {
      return 1;
    }
    if (b.key === "remote-bombs") {
      return -1;
    }
    return (b.remaining ?? 0) - (a.remaining ?? 0);
  });
}

export function formatTimedStatusSummary(player: TimedStatusSource | null, maxVisible: number = 3): string {
  const statuses = buildTimedStatuses(player);
  if (statuses.length === 0) {
    return "";
  }

  const visible = statuses.slice(0, maxVisible).map((status) => {
    if (status.remaining === undefined) {
      return `${status.icon}${status.detail ? ` ${status.detail}` : ""}`;
    }
    const remaining = Math.max(1, Math.ceil(status.remaining));
    return `${status.icon}${status.detail ? ` ${status.detail}` : ""} ${remaining}s`;
  });

  if (statuses.length > maxVisible) {
    visible.push(`+${statuses.length - maxVisible}`);
  }

  return visible.join("  ");
}
