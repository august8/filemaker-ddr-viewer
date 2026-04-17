import { useState, useMemo } from "react";
import type { SearchResult } from "../types/ddr";

export interface SearchFilteringResult {
  activeType: string | null;
  setActiveType: (type: string | null) => void;
  countsByType: Record<string, number>;
  filteredResults: SearchResult[];
}

export function useSearchFiltering(
  results: SearchResult[]
): SearchFilteringResult {
  const [activeType, setActiveType] = useState<string | null>(null);

  const countsByType = useMemo(() => {
    const counts: Record<string, number> = {};
    for (const r of results) {
      counts[r.element_type] = (counts[r.element_type] ?? 0) + 1;
    }
    return counts;
  }, [results]);

  const filteredResults = useMemo(
    () =>
      activeType
        ? results.filter((r) => r.element_type === activeType)
        : results,
    [results, activeType]
  );

  return { activeType, setActiveType, countsByType, filteredResults };
}
