const KEY_ENTER = 13;

import { withElement, setElementDisplay, WebUIManager, ModalData } from "./lib/web";
import { DetonatorGame, GameState } from "./lib/game";
import { DetonatorGameOnline } from "./lib/gameonline";
import { StateMachine } from "./lib/statemachine";

let stateMachine = new StateMachine();
let uiManager: WebUIManager = new WebUIManager();
let gameObj: DetonatorGame | null = null;

let playerName: string;
let offlineMode = false;

// check if nick contains invalid characters. returns true if valid,
// otherwise false.
function validNick(playerNameInput: HTMLInputElement): boolean {
    if (!playerNameInput) {
        return false;
    }

    let regex = /^[\w\s,._:'!^*()=-]+$/;

    if (playerNameInput.value.length > 20) {
        return false;
    }

    return regex.exec(playerNameInput.value) !== null;
}

window.onload = function (): void {
    "use strict";

    // Initialise state machine.
    stateMachine.setInitialState(GameState.Menu);
    stateMachine.addTransition(GameState.Menu, GameState.InitGame, () => {
        startGame();
    });
    stateMachine.addTransition(GameState.RespawnMenu, GameState.InitGame, () => {
        startGame();
    });

    let nickErrorText = document.querySelector("#startMenu .input-error");

    withElement("playerNameInput", (playerNameInput) => {
        withElement("startButton", (btn) => {
            btn.onclick = function () {
                offlineMode = false;

                // check if the nick is valid
                if (validNick(playerNameInput as HTMLInputElement)) {
                    stateMachine.setState(GameState.InitGame);
                } else {
                    if (nickErrorText && nickErrorText instanceof HTMLLabelElement) {
                        nickErrorText.style.display = "inline";
                    }
                }
            };
        });

        playerNameInput.addEventListener("keypress", function (e) {
            let key = e.which || e.keyCode;
            if (key === KEY_ENTER) {
                if (validNick(playerNameInput as HTMLInputElement)) {
                    offlineMode = false;
                    stateMachine.setState(GameState.InitGame);
                } else {
                    if (nickErrorText && nickErrorText instanceof HTMLLabelElement) {
                        nickErrorText.style.display = "inline";
                    }
                }
            }
        });
    });

    withElement("respawnBtn", (respawnBtn) => {
        respawnBtn.onclick = function () {
            stateMachine.setState(GameState.InitGame);
        };
    });

    let display: { [x: string]: string } = {
        extrainfo: "block",
        appbuttons: "block",
        mobileMenu: "none"
    };
    if (uiManager.isMobile()) {
        display = { extrainfo: "none", appbuttons: "none", mobileMenu: "block" };
    }
    setElementDisplay(display);

    uiManager.showMobBanner(false); // don't fade out.

    // Allow user to break out of iframe...
    let isiframe = false;

    // try {
    //     if (!/^https*:\/\/xp\.stevepryde\.com/.test(window.location.href)) {
    //         isiframe = true;
    //     }
    // } catch (err) {
    //     isiframe = true;
    // }

    setElementDisplay({ breakout: isiframe ? "block" : "none" });

    withElement("exitframe", (breakoutBtn) => {
        breakoutBtn.onclick = function () {
            window.open("https://xp.stevepryde.com", "_top");
        };
    });
};

function quitGame(data: ModalData): void {
    uiManager.hideExitPopup();

    if (gameObj) {
        gameObj.cleanup();
        gameObj = null;
    }

    uiManager.showModal(data);
}

function startGame(): void {
    playerName = "unknown";
    withElement("playerNameInput", (playerNameInput) => {
        playerName = (playerNameInput as HTMLInputElement).value.replace(/(<([^>]+)>)/gi, "");
    });

    stateMachine.clear(GameState.InitGame);
    stateMachine.addTransition(GameState.RespawnMenu, GameState.InitGame, () => {
        startGame();
    });
    stateMachine.addTransition(GameState.MainGame, GameState.RespawnMenu, (data: any) => {
        quitGame(data);
    });
    stateMachine.addTransition(GameState.FindServer, GameState.RespawnMenu, (data: any) => {
        quitGame(data);
    });
    gameObj = new DetonatorGameOnline(uiManager, playerName, stateMachine);
}
