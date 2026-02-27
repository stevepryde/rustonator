import { Player } from "./common/player";

export const IMG_PREFIX = "/assets/";

export function withElement(elemId: string, fn: (elem: HTMLElement) => void): void {
  let elem = document.getElementById(elemId);
  if (elem) {
    fn(elem);
  }
}

export function setElementDisplay(displayProperties: { [x: string]: string }): void {
  Object.entries(displayProperties).forEach(([key, value]) => {
    withElement(key, (elem) => {
      elem.style.display = value;
    });
  });
}

export function makep(text: string): string {
  return "<p>" + text + "</p>";
}

export interface ModalData {
  player?: Player;
  message: string;
}

/**
 * Manage all web UI components.
 */
export class WebUIManager {
  constructor() {}

  isMobile(): boolean {
    if (
      /Android|webOS|iPhone|iPad|iPod|BlackBerry|BB|PlayBook|IEMobile|Windows Phone|Kindle|Silk|Opera Mini/i.test(
        navigator.userAgent
      )
    ) {
      // Take the user to a different screen here.
      return true;
    }

    return false;
  }

  isAndroid(): boolean {
    if (/Android/i.test(navigator.userAgent)) {
      return true;
    }

    return false;
  }

  isApple(): boolean {
    if (/(iPhone|iPad)/i.test(navigator.userAgent)) {
      return true;
    }

    return false;
  }

  isChrome(): boolean {
    return /Chrome/.test(navigator.userAgent) && /Google Inc/.test(navigator.vendor);
  }

  showMobBanner(fadeout: boolean = true): void {
    // if (APP_MODE || !isMobile()) {
    //   return;
    // }

    let storeURL = "";
    let storeName = "";
    let storeType = "";

    if (this.isAndroid()) {
      storeURL = "https://play.google.com/store/apps/details?id=io.dstruct";
      storeName = "Google Play Store";
      storeType = "Android";
    } else if (this.isApple()) {
      storeURL = "https://itunes.apple.com/us/app/dstruct.io/id1195514057?mt=8";
      storeName = "the App Store";
      storeType = "iOS";
    } else {
      return;
    }

    let content = "<table border='0' width='100%'><tr>";
    content += "<td><a href='" + storeURL + "'>";
    content += "<img src='/assets/icon.png' width='100px' ";
    content += "height='100px'></a></td>";
    content += "<td>&lt;&lt; Tap the icon to download the ";
    content += storeType + " app from ";
    content += storeName + "!</td></tr></table>";

    withElement("mobbanner", (mobbanner) => {
      mobbanner.innerHTML = content;
      mobbanner.style.display = "block";
    });

    if (fadeout) {
      // Hide the banner after 20 seconds.
      window.setTimeout(this.hideMobBanner, 20000);
    }
  }

  hideMobBanner(): void {
    setElementDisplay({ mobbanner: "none" });
  }

  showLoadingScreen(message: string): void {
    if (!message) {
      message = "Loading...";
    }

    withElement("loadingText", (elem) => {
      elem.innerHTML = makep(message);
    });

    setElementDisplay({
      gameAreaWrapper: "none",
      startMenuWrapper: "none",
      postGameWrapper: "none",
      respawnWrapper: "none",
      loadingWrapper: "block"
    });

    this.showMobBanner();
  }

  showGame(): void {
    setElementDisplay({
      gameAreaWrapper: "block",
      startMenuWrapper: "none",
      postGameWrapper: "none",
      respawnWrapper: "none",
      loadingWrapper: "none"
    });

    this.showMobBanner();
  }

  showExitPopup(): void {
    setElementDisplay({ exitpopup: "block" });
  }

  hideExitPopup(): void {
    setElementDisplay({ exitpopup: "none" });
  }

  showModal(data: ModalData): void {
    let content = "<table border='0' width='80%' style='margin:auto'>";
    content += "<tr><td><img src='" + IMG_PREFIX;
    content += "dsicon200.png' width='100px' height='100px'></td><td>";
    if (data.message) {
      content += makep(data.message);

      // TODO: app mode!
      // if (APP_MODE && AdMob) {
      //   // Don't allow users to go back here which would avoid ads.
      //   blockBackButton = true;
      //   skipAd = false;
      // }
    } else {
      content += makep("You were disconnected. Connectivity issue?");
      content += makep("We apologize for any inconvenience.");
    }

    if (data.player) {
      content += makep("Your final score: " + data.player.score);
    }

    content += "</td></tr></table>";

    withElement("deadReason", (elem) => {
      elem.innerHTML = content;
    });
    setElementDisplay({
      gameAreaWrapper: "none",
      startMenuWrapper: "none",
      postGameWrapper: "block",
      respawnWrapper: "none"
    });

    this.showMobBanner();
  }
}

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

export async function fetchScores(): Promise<ScoreBuckets | null> {
  try {
    const res = await fetch(`${SCORES_URL}/api/scores`);
    if (!res.ok) return null;
    return await res.json();
  } catch {
    return null;
  }
}

function renderScoreTable(entries: ScoreEntry[]): string {
  let html = "<table class='scorebox' style='width:100%;max-width:100%;'>";
  for (let i = 0; i < 10; i++) {
    const entry = entries[i];
    if (entry) {
      html += `<tr><td class='num'>${i + 1}.</td><td class='name'>${escapeHtml(entry.name)}</td><td class='score'>${entry.score}</td></tr>`;
    } else {
      html += `<tr><td class='num'>${i + 1}.</td><td class='name'>---</td><td class='score'>-</td></tr>`;
    }
  }
  html += "</table>";
  return html;
}

function escapeHtml(text: string): string {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

export function renderTodayScores(data: ScoreBuckets, container: HTMLElement): void {
  let html = "<div class='high-scores'>";
  html += "<h3 class='high-scores-title'>TODAY</h3>";
  html += renderScoreTable(data.today);
  html += "</div>";
  container.innerHTML = html;
}

export function renderTopScores(data: ScoreBuckets, container: HTMLElement): void {
  let html = "<div class='high-scores'>";
  html += "<div class='high-scores-tabs'>";
  html += "<button class='high-scores-tab active' data-tab='month'>This Month</button>";
  html += "<button class='high-scores-tab' data-tab='alltime'>All Time</button>";
  html += "</div>";
  html += `<div class='high-scores-panel' data-panel='month'>${renderScoreTable(data.month)}</div>`;
  html += `<div class='high-scores-panel' data-panel='alltime' style='display:none;'>${renderScoreTable(data.all_time)}</div>`;
  html += "</div>";
  container.innerHTML = html;

  container.querySelectorAll(".high-scores-tab").forEach((btn) => {
    btn.addEventListener("click", () => {
      const tab = (btn as HTMLElement).dataset.tab;
      container.querySelectorAll(".high-scores-tab").forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      container.querySelectorAll(".high-scores-panel").forEach((p) => {
        (p as HTMLElement).style.display = (p as HTMLElement).dataset.panel === tab ? "block" : "none";
      });
    });
  });
}

const EMPTY_SCORES: ScoreBuckets = { today: [], month: [], all_time: [] };

export async function fetchAndRenderAllScores(todayId: string, topId: string): Promise<void> {
  const todayContainer = document.getElementById(todayId);
  const topContainer = document.getElementById(topId);
  if (!todayContainer && !topContainer) return;

  // Render empty tables immediately so boxes are always visible.
  if (todayContainer) renderTodayScores(EMPTY_SCORES, todayContainer);
  if (topContainer) renderTopScores(EMPTY_SCORES, topContainer);

  const data = await fetchScores();
  if (data) {
    if (todayContainer) renderTodayScores(data, todayContainer);
    if (topContainer) renderTopScores(data, topContainer);
  }
}
