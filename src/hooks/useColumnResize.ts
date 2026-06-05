import { useRef, useState, useEffect } from "react";

/**
 * テーブルの列幅をドラッグでリサイズするフック。
 * table-layout:fixed の <colgroup> と組み合わせて使用する。
 *
 * ドラッグ開始時に全列幅を DOM から実測して一括固定し、
 * N を広げると N+1 が狭まる（合計幅一定）。最終列のみ単独変更。
 */
export function useColumnResize(count: number) {
  const [widths, setWidths] = useState<(number | undefined)[]>(() =>
    Array(count).fill(undefined)
  );
  const thRefs = useRef<(HTMLTableCellElement | null)[]>(Array(count).fill(null));
  const dragging = useRef<{
    colIndex: number;
    startX: number;
    frozenWidths: number[];
  } | null>(null);

  // 安定したコールバックをマウント時に一度だけ生成（毎レンダーで新規生成しない）
  const thRefCallbacks = useRef<((el: HTMLTableCellElement | null) => void)[]>(
    Array.from({ length: count }, (_, i) => (el: HTMLTableCellElement | null) => {
      thRefs.current[i] = el;
    })
  );

  const setThRef = (colIndex: number) => thRefCallbacks.current[colIndex];

  // アンマウント時にドラッグ中のリスナーを確実に除去する
  const activeListeners = useRef<{
    onMouseMove: ((ev: MouseEvent) => void) | null;
    onMouseUp: (() => void) | null;
  }>({ onMouseMove: null, onMouseUp: null });

  useEffect(() => {
    return () => {
      const { onMouseMove, onMouseUp } = activeListeners.current;
      if (onMouseMove) document.removeEventListener("mousemove", onMouseMove);
      if (onMouseUp) document.removeEventListener("mouseup", onMouseUp);
    };
  }, []);

  const getResizeHandleProps = (colIndex: number) => ({
    onMouseDown: (e: React.MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();

      // 全列の実測幅を取得して一括固定（auto列の再配分を防ぐ）
      const frozenWidths = thRefs.current.map(
        (th) => th?.offsetWidth ?? 100
      ) as number[];
      setWidths([...frozenWidths]);

      dragging.current = { colIndex, startX: e.clientX, frozenWidths };

      const onMouseMove = (ev: MouseEvent) => {
        if (!dragging.current) return;
        const { colIndex: idx, frozenWidths } = dragging.current;
        const delta = ev.clientX - dragging.current.startX;
        const newWidth = Math.max(40, frozenWidths[idx] + delta);
        const actualDelta = newWidth - frozenWidths[idx];

        // 常に frozenWidths を基準に計算（React state の非同期更新に依存しない）
        setWidths(() => {
          const next = [...frozenWidths];
          next[idx] = newWidth;
          if (idx + 1 < frozenWidths.length) {
            next[idx + 1] = Math.max(40, frozenWidths[idx + 1] - actualDelta);
          }
          return next;
        });
      };

      const onMouseUp = () => {
        dragging.current = null;
        activeListeners.current.onMouseMove = null;
        activeListeners.current.onMouseUp = null;
        document.removeEventListener("mousemove", onMouseMove);
        document.removeEventListener("mouseup", onMouseUp);
      };

      activeListeners.current.onMouseMove = onMouseMove;
      activeListeners.current.onMouseUp = onMouseUp;
      document.addEventListener("mousemove", onMouseMove);
      document.addEventListener("mouseup", onMouseUp);
    },
  });

  return { widths, setThRef, getResizeHandleProps };
}
