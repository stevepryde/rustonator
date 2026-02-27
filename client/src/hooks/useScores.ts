import { useEffect, useState } from "react";

export interface ScoreEntry {
  name: string;
  score: number;
  timestamp: number;
}

export interface ScoreBuckets {
  today: ScoreEntry[];
  month: ScoreEntry[];
  all_time: ScoreEntry[];
}

const SCORES_URL = import.meta.env.VITE_SCORES_URL || "";

export function useScores() {
  const [scores, setScores] = useState<ScoreBuckets | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const res = await fetch(`${SCORES_URL}/api/scores`);
        if (!res.ok) return;
        const data: ScoreBuckets = await res.json();
        if (!cancelled) setScores(data);
      } catch {
        // ignore
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    load();
    return () => { cancelled = true; };
  }, []);

  return { scores, loading };
}
