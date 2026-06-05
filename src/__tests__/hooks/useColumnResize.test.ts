import { renderHook, act } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { useColumnResize } from "../../hooks/useColumnResize";

describe("useColumnResize", () => {
  it("初期状態で widths がすべて undefined", () => {
    const { result } = renderHook(() => useColumnResize(3));
    expect(result.current.widths).toEqual([undefined, undefined, undefined]);
  });

  it("setThRef が th 要素を ref に登録する", () => {
    const { result } = renderHook(() => useColumnResize(2));
    const el = document.createElement("th");
    Object.defineProperty(el, "offsetWidth", { value: 120, configurable: true });

    act(() => {
      result.current.setThRef(0)(el);
    });

    // ref 登録後に onMouseDown を呼ぶと frozenWidths に反映される
    const el2 = document.createElement("th");
    Object.defineProperty(el2, "offsetWidth", { value: 80, configurable: true });
    act(() => {
      result.current.setThRef(1)(el2);
    });

    act(() => {
      result.current.getResizeHandleProps(0).onMouseDown({
        preventDefault: () => {},
        stopPropagation: () => {},
        clientX: 0,
      } as unknown as React.MouseEvent);
    });

    // onMouseDown だけでは widths が frozenWidths に初期化される
    expect(result.current.widths).toEqual([120, 80]);
  });

  it("mousemove で対象列が広がり隣接列が縮む", () => {
    const { result } = renderHook(() => useColumnResize(2));

    const th0 = document.createElement("th");
    Object.defineProperty(th0, "offsetWidth", { value: 100, configurable: true });
    const th1 = document.createElement("th");
    Object.defineProperty(th1, "offsetWidth", { value: 200, configurable: true });

    act(() => {
      result.current.setThRef(0)(th0);
      result.current.setThRef(1)(th1);
    });

    act(() => {
      result.current.getResizeHandleProps(0).onMouseDown({
        preventDefault: () => {},
        stopPropagation: () => {},
        clientX: 50,
      } as unknown as React.MouseEvent);
    });

    act(() => {
      document.dispatchEvent(new MouseEvent("mousemove", { clientX: 70 }));
    });

    // +20px ドラッグ: col0 → 120, col1 → 180
    expect(result.current.widths[0]).toBe(120);
    expect(result.current.widths[1]).toBe(180);

    act(() => {
      document.dispatchEvent(new MouseEvent("mouseup"));
    });
  });

  it("最小幅 40px を下回らない", () => {
    const { result } = renderHook(() => useColumnResize(2));

    const th0 = document.createElement("th");
    Object.defineProperty(th0, "offsetWidth", { value: 50, configurable: true });
    const th1 = document.createElement("th");
    Object.defineProperty(th1, "offsetWidth", { value: 50, configurable: true });

    act(() => {
      result.current.setThRef(0)(th0);
      result.current.setThRef(1)(th1);
    });

    act(() => {
      result.current.getResizeHandleProps(0).onMouseDown({
        preventDefault: () => {},
        stopPropagation: () => {},
        clientX: 50,
      } as unknown as React.MouseEvent);
    });

    // -100px ドラッグ → 最小幅 40 にクランプされる
    act(() => {
      document.dispatchEvent(new MouseEvent("mousemove", { clientX: -50 }));
    });

    expect(result.current.widths[0]).toBe(40);

    act(() => {
      document.dispatchEvent(new MouseEvent("mouseup"));
    });
  });

  it("最終列のみリサイズ時は隣接列が変化しない", () => {
    const { result } = renderHook(() => useColumnResize(2));

    const th0 = document.createElement("th");
    Object.defineProperty(th0, "offsetWidth", { value: 100, configurable: true });
    const th1 = document.createElement("th");
    Object.defineProperty(th1, "offsetWidth", { value: 100, configurable: true });

    act(() => {
      result.current.setThRef(0)(th0);
      result.current.setThRef(1)(th1);
    });

    act(() => {
      result.current.getResizeHandleProps(1).onMouseDown({
        preventDefault: () => {},
        stopPropagation: () => {},
        clientX: 50,
      } as unknown as React.MouseEvent);
    });

    act(() => {
      document.dispatchEvent(new MouseEvent("mousemove", { clientX: 80 }));
    });

    // 最終列(idx=1)のみ変化。col0 は frozenWidths のまま
    expect(result.current.widths[0]).toBe(100);
    expect(result.current.widths[1]).toBe(130);

    act(() => {
      document.dispatchEvent(new MouseEvent("mouseup"));
    });
  });
});
