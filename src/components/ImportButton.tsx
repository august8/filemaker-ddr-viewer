import { open } from "@tauri-apps/plugin-dialog";
import { useImportSolution } from "../hooks/useTauriCommand";
import { Spinner } from "./Spinner";

export function ImportButton() {
  const { mutate, isPending, isError, error } = useImportSolution();

  const handleClick = async () => {
    const path = await open({
      title: "概要.xml を選択",
      filters: [{ name: "DDR Summary", extensions: ["xml"] }],
    });
    if (typeof path === "string") {
      mutate(path);
    }
  };

  return (
    <div className="flex items-center gap-2">
      <button
        onClick={handleClick}
        disabled={isPending}
        className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
      >
        {isPending ? (
          <span className="flex items-center gap-1.5">
            <Spinner className="w-3.5 h-3.5" />
            インポート中...
          </span>
        ) : "DDR をインポート"}
      </button>
      {isError && (
        <span className="text-sm text-red-500">{String(error)}</span>
      )}
    </div>
  );
}
