interface LoadingScreenProps {
  message: string;
}

export default function LoadingScreen({ message }: LoadingScreenProps) {
  return (
    <div className="fixed inset-0 z-10 overflow-auto p-2.5 retro-dotpattern flex items-start justify-center">
      <div className="bg-gray-500 mt-[10%] p-2.5 w-[70%] text-center rounded box-border">
        <p className="text-xl font-bold text-black">{message || "Loading..."}</p>
      </div>
    </div>
  );
}
