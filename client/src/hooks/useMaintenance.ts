import { useEffect, useState } from "react";

const SCORES_URL = import.meta.env.VITE_SCORES_URL || "";

export interface MaintenanceState {
  enabled: boolean;
  message: string;
}

const DEFAULT_STATE: MaintenanceState = {
  enabled: false,
  message: "Under maintenance. Please try again later.",
};

export function useMaintenance() {
  const [maintenance, setMaintenance] = useState<MaintenanceState>(DEFAULT_STATE);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const res = await fetch(`${SCORES_URL}/api/maintenance`);
        if (!res.ok) return;
        const data: MaintenanceState = await res.json();
        if (!cancelled) {
          setMaintenance({
            enabled: !!data.enabled,
            message: data.message || DEFAULT_STATE.message,
          });
        }
      } catch {
        // ignore
      }
    }

    load();
    return () => {
      cancelled = true;
    };
  }, []);

  return maintenance;
}
