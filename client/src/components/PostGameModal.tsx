interface PostGameModalProps {
  reason: string;
  score: number;
  onRespawn: () => void;
}

export default function PostGameModal({ reason, score, onRespawn }: PostGameModalProps) {
  const message = reason || "You were disconnected. Connectivity issue? We apologize for any inconvenience.";

  return (
    <div className="fixed inset-0 z-10 overflow-auto p-2.5 retro-dotpattern">
      <div className="bg-gray-500 mx-auto mt-[10%] p-2.5 w-[70%] text-center rounded box-border">
        <table className="w-4/5 mx-auto">
          <tbody>
            <tr>
              <td>
                <img
                  src="/assets/dsicon200.png"
                  alt="game icon"
                  className="w-[100px] h-[100px]"
                />
              </td>
              <td>
                <p className="text-xl font-bold text-black p-0">{message}</p>
                {score > 0 && (
                  <p className="text-xl font-bold text-black p-0">
                    Your final score: {score}
                  </p>
                )}
              </td>
            </tr>
          </tbody>
        </table>
        <hr className="my-2" />
        <button
          type="button"
          onClick={onRespawn}
          className="btn-solid w-[30%] h-10 text-lg cursor-pointer"
        >
          Respawn
        </button>
      </div>
    </div>
  );
}
