import { useRef, useState, useCallback, useMemo, useEffect } from "react";
import dagre from "dagre";
import { useTableOccurrenceList, useRelationshipList } from "../../hooks/useTauriCommand";
import { useAppStore } from "../../stores/appStore";
import type { TableOccurrenceRow, RelationshipRow } from "../../types/ddr";
import { Spinner } from "../Spinner";

interface Props {
  projectId: number;
}

const NODE_HEIGHT = 52;
const LABEL_FONT_SIZE = 9;
const LABEL_PAD_X   = 7;
const LABEL_PAD_Y   = 4;
const LABEL_LINE_H  = 14;

function labelFont(fontSize: number) { return `${fontSize - 2}px monospace`; }
function nodeTitleFont(fontSize: number) { return `bold ${fontSize + 1}px sans-serif`; }
// canvas.measureText() でテキスト幅を正確に計測
let _ctx: CanvasRenderingContext2D | null = null;
function measureText(text: string, font: string): number {
  if (typeof document === "undefined") return text.length * 7;
  if (!_ctx) _ctx = document.createElement("canvas").getContext("2d");
  if (!_ctx) return text.length * 7;
  _ctx.font = font;
  return _ctx.measureText(text).width;
}

function calcNodeWidth(name: string, fontSize: number): number {
  return Math.max(120, Math.ceil(measureText(name, nodeTitleFont(fontSize))) + 40);
}

interface LayoutNode {
  id: string;
  x: number;
  y: number;
  w: number;
  occurrenceName: string;
  baseTableName: string;
  /** base_table_name が空 = 外部データソース参照 */
  isExternal: boolean;
}

interface PredicateInfo {
  leftField: string;
  operator: string;
  rightField: string;
}

interface LayoutEdge {
  id: number;
  points: Array<{ x: number; y: number }>;
  predicates: PredicateInfo[];
  leftTable: string;
  rightTable: string;
}

function buildEdgePath(points: Array<{ x: number; y: number }>): string {
  if (points.length < 2) return "";
  const [first, ...rest] = points;
  return `M ${first.x} ${first.y} ` + rest.map((p) => `L ${p.x} ${p.y}`).join(" ");
}

function tickPath(p: { x: number; y: number }, dir: { x: number; y: number }, size = 5): string {
  const len = Math.sqrt(dir.x * dir.x + dir.y * dir.y) || 1;
  const nx = -dir.y / len;
  const ny = dir.x / len;
  return `M ${p.x + nx * size} ${p.y + ny * size} L ${p.x - nx * size} ${p.y - ny * size}`;
}

function buildLayout(
  tableOccurrences: TableOccurrenceRow[],
  relationships: RelationshipRow[],
  fontSize: number
): { nodes: LayoutNode[]; edges: LayoutEdge[]; width: number; height: number } {
  const g = new dagre.graphlib.Graph({ multigraph: true });
  g.setGraph({ rankdir: "LR", marginx: 40, marginy: 40, nodesep: 60, ranksep: 160, acyclicer: "greedy" });
  g.setDefaultEdgeLabel(() => ({}));

  for (const to of tableOccurrences) {
    const w = calcNodeWidth(to.occurrence_name, fontSize);
    g.setNode(to.occurrence_name, { label: to.occurrence_name, width: w, height: NODE_HEIGHT });
  }

  const nodeIds = new Set(tableOccurrences.map((to) => to.occurrence_name));
  for (const rel of relationships) {
    if (nodeIds.has(rel.left_table) && nodeIds.has(rel.right_table)) {
      g.setEdge(rel.left_table, rel.right_table, {}, String(rel.id));
    }
  }

  dagre.layout(g);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const graphMeta = g.graph() as any;

  const nodes: LayoutNode[] = g.nodes().map((nodeId) => {
    const n  = g.node(nodeId);
    const to = tableOccurrences.find((t) => t.occurrence_name === nodeId)!;
    return { id: nodeId, x: n.x, y: n.y, w: n.width, occurrenceName: to.occurrence_name, baseTableName: to.base_table_name, isExternal: to.source_file !== "" };
  });

  const edges: LayoutEdge[] = g.edges().map((e) => {
    const edge = g.edge(e);
    const rel  = relationships.find((r) => String(r.id) === e.name);
    return {
      id:         rel?.id ?? 0,
      points:     edge?.points ?? [],
      predicates: rel?.predicates.map((p) => ({ leftField: p.left_field, operator: p.operator, rightField: p.right_field })) ?? [],
      leftTable:  rel?.left_table  ?? "",
      rightTable: rel?.right_table ?? "",
    };
  });

  return { nodes, edges, width: graphMeta.width ?? 0, height: graphMeta.height ?? 0 };
}

// ポップアップ（ホバー追従 / クリックで固定の両方に使う）
function EdgePopup({
  edge,
  pos,
  pinned = false,
  onClose,
  fontSize = 14,
}: {
  edge: LayoutEdge;
  pos: { x: number; y: number };
  pinned?: boolean;
  onClose?: () => void;
  fontSize?: number;
}) {
  const COL_GAP = 10;
  const CLOSE_SIZE = 14; // ピン時の × ボタン領域
  const lf = labelFont(fontSize);

  const leftW  = edge.predicates.length > 0 ? Math.max(...edge.predicates.map((p) => measureText(p.leftField,  lf))) : 0;
  const opW    = edge.predicates.length > 0 ? Math.max(...edge.predicates.map((p) => measureText(p.operator,   lf))) : 0;
  const rightW = edge.predicates.length > 0 ? Math.max(...edge.predicates.map((p) => measureText(p.rightField, lf))) : 0;

  const popupHeaderFont = `bold ${fontSize - 2}px sans-serif`;
  const headerText = `${edge.leftTable} → ${edge.rightTable}`;
  const headerW    = measureText(headerText, popupHeaderFont) + (pinned ? CLOSE_SIZE : 0);
  const predicateW = leftW + COL_GAP + opW + COL_GAP + rightW;
  const boxW = Math.max(headerW, predicateW) + LABEL_PAD_X * 2;
  const boxH = edge.predicates.length * LABEL_LINE_H + LABEL_PAD_Y * 2 + 18;

  const OFFSET = 14;
  const bx = pos.x + OFFSET;
  const by = pos.y - OFFSET - boxH;

  const borderColor = pinned ? "#3b82f6" : "#64748b";
  const headerBg    = pinned ? "#eff6ff" : "#f1f5f9";

  return (
    <g style={{ pointerEvents: pinned ? "all" : "none" }}>
      <rect x={bx + 2} y={by + 2} width={boxW} height={boxH} rx={4} fill="rgba(0,0,0,0.15)" />
      <rect x={bx} y={by} width={boxW} height={boxH} rx={4} fill="white" stroke={borderColor} strokeWidth={pinned ? 1.5 : 1} />
      {/* ヘッダー */}
      <rect x={bx} y={by} width={boxW} height={16} rx={4} fill={headerBg} />
      <rect x={bx} y={by + 12} width={boxW} height={4} fill={headerBg} />
      <text x={bx + LABEL_PAD_X} y={by + 11} textAnchor="start" fontSize={fontSize - 2} fill="#475569" fontWeight="bold">
        {edge.leftTable} → {edge.rightTable}
      </text>
      {/* ピン固定時の × ボタン */}
      {pinned && onClose && (
        <g onClick={onClose} style={{ cursor: "pointer" }}>
          <rect x={bx + boxW - CLOSE_SIZE} y={by} width={CLOSE_SIZE} height={16} rx={4} fill="transparent" />
          <text x={bx + boxW - CLOSE_SIZE / 2} y={by + 11} textAnchor="middle" fontSize={10} fill="#94a3b8">×</text>
        </g>
      )}
      <line x1={bx} y1={by + 16} x2={bx + boxW} y2={by + 16} stroke="#e2e8f0" strokeWidth={1} />
      {/* 述語行 */}
      {edge.predicates.map((pred, i) => {
        const rowY = by + 16 + LABEL_PAD_Y + i * LABEL_LINE_H + LABEL_LINE_H * 0.72;
        return (
          <g key={i}>
            {i > 0 && (
              <line x1={bx + LABEL_PAD_X} y1={by + 16 + LABEL_PAD_Y + i * LABEL_LINE_H} x2={bx + boxW - LABEL_PAD_X} y2={by + 16 + LABEL_PAD_Y + i * LABEL_LINE_H} stroke="#f1f5f9" strokeWidth={1} />
            )}
            <text x={bx + LABEL_PAD_X}                                    y={rowY} fontSize={LABEL_FONT_SIZE} fontFamily="monospace" fill="#1e40af" textAnchor="start" style={{ fontSize: `${fontSize - 2}px` }}>{pred.leftField}</text>
            <text x={bx + LABEL_PAD_X + leftW + COL_GAP}                  y={rowY} fontSize={LABEL_FONT_SIZE} fontFamily="monospace" fill="#64748b" textAnchor="start" style={{ fontSize: `${fontSize - 2}px` }}>{pred.operator}</text>
            <text x={bx + LABEL_PAD_X + leftW + COL_GAP + opW + COL_GAP}  y={rowY} fontSize={LABEL_FONT_SIZE} fontFamily="monospace" fill="#166534" textAnchor="start" style={{ fontSize: `${fontSize - 2}px` }}>{pred.rightField}</text>
          </g>
        );
      })}
    </g>
  );
}

interface Transform { x: number; y: number; scale: number; }

export function RelationshipGraphPanel({ projectId }: Props) {
  const { data: tableOccurrences = [], isLoading: toLoading } = useTableOccurrenceList(projectId);
  const { data: relationships    = [], isLoading: relLoading } = useRelationshipList(projectId);
  const isLoading = toLoading || relLoading;
  const fontSize = useAppStore((s) => s.fontSize);

  const [searchTerm,    setSearchTerm]    = useState("");
  const [hoveredEdgeId, setHoveredEdgeId] = useState<number | null>(null);
  const [hoverPos,      setHoverPos]      = useState<{ x: number; y: number } | null>(null);
  const [pinnedEdge,    setPinnedEdge]    = useState<{ id: number; pos: { x: number; y: number } } | null>(null);
  const query = searchTerm.trim().toLowerCase();

  const { nodes, edges, width, height } = useMemo(() => {
    if (isLoading || tableOccurrences.length === 0) return { nodes: [], edges: [], width: 0, height: 0 };
    return buildLayout(tableOccurrences, relationships, fontSize);
  }, [tableOccurrences, relationships, isLoading, fontSize]);

  const [transform, setTransform] = useState<Transform>({ x: 20, y: 20, scale: 1 });

  /** clientX/Y → SVG座標に変換 */
  const toSvgCoords = useCallback((clientX: number, clientY: number) => {
    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect) return null;
    return {
      x: (clientX - rect.left  - transform.x) / transform.scale,
      y: (clientY - rect.top   - transform.y) / transform.scale,
    };
  }, [transform]);

  const isPanning = useRef(false);
  const lastPos   = useRef({ x: 0, y: 0 });
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const handler = (e: WheelEvent) => {
      e.preventDefault();
      const delta = e.deltaY > 0 ? 0.9 : 1.1;
      setTransform((prev) => ({ ...prev, scale: Math.min(3, Math.max(0.1, prev.scale * delta)) }));
    };
    el.addEventListener("wheel", handler, { passive: false });
    return () => el.removeEventListener("wheel", handler);
  }, []);

  const onMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return;
    isPanning.current = true;
    lastPos.current = { x: e.clientX, y: e.clientY };
    e.preventDefault();
  }, []);

  const onMouseMove = useCallback((e: React.MouseEvent) => {
    if (!isPanning.current) return;
    const dx = e.clientX - lastPos.current.x;
    const dy = e.clientY - lastPos.current.y;
    lastPos.current = { x: e.clientX, y: e.clientY };
    setTransform((prev) => ({ ...prev, x: prev.x + dx, y: prev.y + dy }));
  }, []);

  const onMouseUp  = useCallback(() => { isPanning.current = false; }, []);
  const resetView  = useCallback(() => setTransform({ x: 20, y: 20, scale: 1 }), []);

  if (isLoading)                    return <div className="flex-1 flex items-center justify-center gap-2 text-gray-400 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  if (tableOccurrences.length === 0) return <div className="flex-1 flex items-center justify-center text-gray-400 text-sm">テーブルオカレンスなし</div>;

  const svgWidth  = Math.max(width  + 100, 400);
  const svgHeight = Math.max(height + 100, 300);
  const hoveredEdge  = edges.find((e) => e.id === hoveredEdgeId) ?? null;
  const activeEdgeId = pinnedEdge?.id ?? hoveredEdgeId;
  const activeEdge   = edges.find((e) => e.id === activeEdgeId) ?? null;

  return (
    <div className="flex flex-col h-full">
      <div className="px-4 py-2 shrink-0 flex items-center gap-3 border-b border-gray-200 bg-white">
        <h2 className="text-base font-bold text-gray-800">リレーショングラフ</h2>
        <span className="text-xs text-gray-400">
          {tableOccurrences.length} TO / {relationships.length} リレーション
        </span>
        <input
          type="text"
          placeholder="オカレンス名で検索..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="w-48 px-2 py-1 text-xs border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-blue-400"
          data-testid="graph-search"
        />
        <span className="text-xs text-gray-400 ml-2">エッジにカーソルを合わせると詳細を表示</span>
        <span className="flex items-center gap-1 text-xs text-purple-600 ml-2">
          <svg width="14" height="10"><rect x="1" y="1" width="12" height="8" rx="2" fill="#fdf4ff" stroke="#c084fc" strokeWidth="1.5" strokeDasharray="4 2"/></svg>
          外部参照
        </span>
        <button onClick={resetView} className="ml-auto text-xs px-2 py-1 rounded border border-gray-300 text-gray-600 hover:bg-gray-50">
          リセット
        </button>
      </div>

      <div
        ref={containerRef}
        className="flex-1 overflow-hidden relative bg-gray-50 select-none"
        style={{ cursor: isPanning.current ? "grabbing" : "grab" }}
        onMouseDown={onMouseDown}
        onMouseMove={onMouseMove}
        onMouseUp={onMouseUp}
        onMouseLeave={onMouseUp}
        data-testid="graph-canvas"
      >
        <svg
          width={svgWidth}
          height={svgHeight}
          style={{ transform: `translate(${transform.x}px, ${transform.y}px) scale(${transform.scale})`, transformOrigin: "0 0", overflow: "visible" }}
        >
          {/* キャンバス背景クリックでピン解除（最背面に配置） */}
          <rect
            x={-9999} y={-9999} width={99999} height={99999}
            fill="transparent"
            onClick={() => setPinnedEdge(null)}
          />

          {/* エッジ */}
          {edges.map((edge) => {
            if (edge.points.length < 2) return null;
            const d = buildEdgePath(edge.points);
            if (!d) return null;

            const edgeRel     = relationships.find((r) => r.id === edge.id);
            const leftMatches  = !!(query && edgeRel && edgeRel.left_table.toLowerCase().includes(query));
            const rightMatches = !!(query && edgeRel && edgeRel.right_table.toLowerCase().includes(query));
            const edgeDimmed   = !!(query && !leftMatches && !rightMatches);
            const isHovered    = edge.id === hoveredEdgeId || edge.id === pinnedEdge?.id;

            const pts        = edge.points;
            const firstPt    = pts[0];
            const secondPt   = pts[1];
            const lastPt     = pts[pts.length - 1];
            const prevLastPt = pts[pts.length - 2];
            const startDir   = { x: secondPt.x - firstPt.x,  y: secondPt.y - firstPt.y };
            const endDir     = { x: lastPt.x - prevLastPt.x, y: lastPt.y - prevLastPt.y };

            const strokeColor = edgeDimmed ? "#e2e8f0" : isHovered ? "#3b82f6" : "#94a3b8";
            const strokeWidth = isHovered ? 2.5 : 1.5;

            return (
              <g
                key={edge.id}
                data-testid="graph-edge"
                onMouseEnter={(e) => {
                  setHoveredEdgeId(edge.id);
                  const p = toSvgCoords(e.clientX, e.clientY);
                  if (p) setHoverPos(p);
                }}
                onMouseMove={(e) => {
                  if (isPanning.current) return;
                  const p = toSvgCoords(e.clientX, e.clientY);
                  if (p) setHoverPos(p);
                }}
                onMouseLeave={() => {
                  setHoveredEdgeId(null);
                  setHoverPos(null);
                }}
                onClick={(e) => {
                  e.stopPropagation();
                  const p = toSvgCoords(e.clientX, e.clientY);
                  if (!p) return;
                  // 同じエッジを再クリック → ピン解除、別エッジ → ピン固定
                  setPinnedEdge((prev) =>
                    prev?.id === edge.id ? null : { id: edge.id, pos: p }
                  );
                }}
                style={{ cursor: "pointer" }}
              >
                {/* ヒットエリア（細いエッジを拾いやすくする透明の太線） */}
                <path d={d} fill="none" stroke="transparent" strokeWidth={12} />
                {/* 実線 */}
                <path d={d} fill="none" stroke={strokeColor} strokeWidth={strokeWidth} />
                <path d={tickPath(firstPt, startDir)} stroke={strokeColor} strokeWidth={strokeWidth} fill="none" />
                <path d={tickPath(lastPt,  endDir)}   stroke={strokeColor} strokeWidth={strokeWidth} fill="none" />
              </g>
            );
          })}

          {/* ノード */}
          {nodes.map((node) => {
            const matches = !!(query && (node.occurrenceName.toLowerCase().includes(query) || node.baseTableName.toLowerCase().includes(query)));
            const dimmed  = !!(query && !matches);
            const highlighted = activeEdge && (activeEdge.leftTable === node.id || activeEdge.rightTable === node.id);

            return (
              <g key={node.id} transform={`translate(${node.x - node.w / 2}, ${node.y - NODE_HEIGHT / 2})`} data-testid="graph-node" opacity={dimmed ? 0.3 : 1}>
                <rect
                  width={node.w}
                  height={NODE_HEIGHT}
                  rx={4}
                  fill={matches ? "#fffbeb" : highlighted ? "#eff6ff" : node.isExternal ? "#fdf4ff" : "white"}
                  stroke={matches ? "#f59e0b" : highlighted ? "#3b82f6" : node.isExternal ? "#c084fc" : "#93c5fd"}
                  strokeWidth={matches || highlighted ? 2.5 : node.isExternal ? 1.5 : 1.5}
                  strokeDasharray={node.isExternal ? "4 2" : undefined}
                />
                <text x={node.w / 2} y={20} textAnchor="middle" fontSize={fontSize + 1} fontWeight="bold" fill={node.isExternal ? "#7e22ce" : "#1e3a5f"}>
                  {node.occurrenceName}
                  <title>{node.occurrenceName}</title>
                </text>
                <text x={node.w / 2} y={36} textAnchor="middle" fontSize={fontSize - 1} fill={node.isExternal ? "#a855f7" : "#6b7280"}>
                  {node.isExternal ? `[${(tableOccurrences.find(t => t.occurrence_name === node.id)?.source_file) ?? "外部"}]` : node.baseTableName}
                </text>
              </g>
            );
          })}

          {/* ホバーポップアップ（ピン固定中のエッジは除く） */}
          {hoveredEdge && hoverPos && hoveredEdge.id !== pinnedEdge?.id && (
            <EdgePopup edge={hoveredEdge} pos={hoverPos} fontSize={fontSize} />
          )}

          {/* ピン固定ポップアップ（最前面） */}
          {pinnedEdge && (() => {
            const pe = edges.find((e) => e.id === pinnedEdge.id);
            return pe ? (
              <EdgePopup
                edge={pe}
                pos={pinnedEdge.pos}
                pinned
                onClose={() => setPinnedEdge(null)}
                fontSize={fontSize}
              />
            ) : null;
          })()}
        </svg>
      </div>
    </div>
  );
}
