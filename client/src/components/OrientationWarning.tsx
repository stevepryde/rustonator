import { useEffect, useState } from "react";

export default function OrientationWarning() {
  const [portrait, setPortrait] = useState(false);

  useEffect(() => {
    function check() {
      setPortrait(window.innerWidth < window.innerHeight);
    }
    check();
    window.addEventListener("resize", check);
    return () => window.removeEventListener("resize", check);
  }, []);

  if (!portrait) return null;

  return (
    <div className="absolute top-[50px] left-0 w-[400px] z-30 bg-black/80 text-white p-12 text-2xl font-bold md:hidden">
      <p>This game must be played in landscape orientation. Please turn your device 90 degrees.</p>
    </div>
  );
}
