import { useCallback, useState, type KeyboardEvent } from "react";
import { useScores } from "../hooks/useScores";
import CharacterSelect from "./CharacterSelect";
import ScoreBoard from "./ScoreBoard";
import Footer from "./Footer";

const NAME_REGEX = /^[\w\s,._:'!^*()=-]+$/;

function validNick(value: string): boolean {
  if (value.length === 0 || value.length > 30) return false;
  return NAME_REGEX.test(value);
}

interface StartMenuProps {
  playerName: string;
  character: string;
  onNameChange: (name: string) => void;
  onCharacterChange: (character: string) => void;
  onStart: () => void;
}

export default function StartMenu({
  playerName,
  character,
  onNameChange,
  onCharacterChange,
  onStart,
}: StartMenuProps) {
  const { scores } = useScores();
  const [showError, setShowError] = useState(false);

  const tryStart = useCallback(() => {
    if (validNick(playerName)) {
      setShowError(false);
      onStart();
    } else {
      setShowError(true);
    }
  }, [playerName, onStart]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Enter") tryStart();
    },
    [tryStart]
  );

  return (
    <div
      className="w-full min-h-full p-2.5 overflow-y-auto retro-dotpattern"
      style={{
        backgroundImage: "url(/assets/screen1_small.jpg)",
        backgroundSize: "cover",
      }}
    >
      {/* Title */}
      <div className="mx-auto text-center pt-5 w-1/2">
        <img src="/assets/dstitle.png" alt="Detonator" className="w-[360px] inline-block" />
      </div>

      {/* Main controls box */}
      <div className="green-border-box mt-5 w-[400px] max-w-[90vw] text-center text-white text-sm">
        <a href="/instructions.html">CLICK HERE FOR INSTRUCTIONS</a>
        <br />
        <br />
        <p className="text-xs text-center">
          This page may take a few seconds to load the first time
        </p>

        <CharacterSelect selected={character} onSelect={onCharacterChange} />

        <input
          type="text"
          autoFocus
          placeholder="Enter your name here"
          maxLength={30}
          value={playerName}
          onChange={(e) => onNameChange(e.target.value)}
          onKeyDown={handleKeyDown}
          className="w-full text-center p-2.5 border border-gray-300 rounded mb-2.5 outline-none transition-shadow focus:border-gray-400 focus:shadow-[0_0_3px_1px_#ddd] box-border bg-black/50 text-white placeholder-gray-400"
        />

        {showError && (
          <b className="text-retro-error text-xs block mb-2">
            Valid Characters: a-z A-Z 0-9 , . _ : ' ! ^ * ( ) = -<br />
            Maximum length: 30 characters (including spaces)
          </b>
        )}

        <button
          type="button"
          onClick={tryStart}
          className="btn-solid w-full max-w-[400px] mt-2.5 font-bold"
        >
          PLAY
        </button>

        {/* Desktop-only controls info */}
        <div className="hidden md:block mt-6">
          <b>Desktop Controls</b>
          <hr className="border-retro-green-dark/50 my-2" />
          <b>Move</b> - Arrow keys
          <br />
          <b>Bomb</b> - Space bar
          <br />
          <br />
          <a href="/glossary.html">Power-up item reference</a>
        </div>
      </div>

      {/* Score boards */}
      <div className="fixed bottom-5 left-5 w-[360px] z-20 green-border-box hidden lg:block">
        <ScoreBoard scores={scores} variant="today" />
      </div>
      <div className="fixed bottom-5 right-5 w-[360px] z-20 green-border-box hidden lg:block">
        <ScoreBoard scores={scores} variant="top" />
      </div>

      <Footer />
    </div>
  );
}
