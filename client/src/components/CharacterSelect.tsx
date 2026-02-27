const CHARACTERS = ["p1", "p2", "p3", "p4"] as const;

interface CharacterSelectProps {
  selected: string;
  onSelect: (character: string) => void;
}

export default function CharacterSelect({ selected, onSelect }: CharacterSelectProps) {
  return (
    <div className="flex justify-center gap-3 my-4">
      {CHARACTERS.map((char) => (
        <button
          key={char}
          type="button"
          onClick={() => onSelect(char)}
          className={`w-16 h-16 rounded border-2 cursor-pointer bg-black/50 transition-all flex items-center justify-center ${
            selected === char
              ? "border-retro-green-glow shadow-[0_0_10px_var(--color-retro-green-glow)]"
              : "border-retro-green-dark/50 hover:border-retro-green"
          }`}
        >
          <div
            className="w-8 h-8 image-rendering-pixelated"
            style={{
              backgroundImage: `url(/assets/${char}icon.png)`,
              backgroundSize: "contain",
              backgroundRepeat: "no-repeat",
              backgroundPosition: "center",
            }}
          />
        </button>
      ))}
    </div>
  );
}
