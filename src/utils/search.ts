// @ts-ignore
import init, { NucleoSearch } from '../wasm/nucleo_wasm.js';

export interface MatchResult {
  line_index: number;
  score: number;
  highlights: number[];
  view_id: string;
}

export interface SearchResult {
  matches: MatchResult[];
  total_count: number;
  query: string;
  elapsed_ms: number;
}

let searchEngine: NucleoSearch | null = null;

async function getSearchEngine(): Promise<NucleoSearch> {
  if (!searchEngine) {
    await init();
    searchEngine = new NucleoSearch();
  }
  return searchEngine;
}

export async function performFuzzySearch(
  query: string,
  viewLines: { viewId: string; lines: string[] }[],
  caseSensitive: boolean,
  limit: number = 100
): Promise<SearchResult> {
  const startTime = performance.now();
  const engine = await getSearchEngine();

  // Combine all lines with view info
  const allLines: { viewId: string; lineIndex: number; text: string }[] = [];
  viewLines.forEach(({ viewId, lines }) => {
    lines.forEach((line, index) => {
      allLines.push({ viewId, lineIndex: index, text: line });
    });
  });

  // Perform search
  const rawResults = engine.search(
    query,
    allLines.map(l => l.text),
    caseSensitive,
    limit
  );

  // Map results back to view info
  const matches: MatchResult[] = rawResults.map((r: any) => ({
    line_index: allLines[r.line_index].lineIndex,
    score: r.score,
    highlights: r.highlights,
    view_id: allLines[r.line_index].viewId,
  }));

  const elapsed = performance.now() - startTime;

  return {
    matches,
    total_count: matches.length,
    query,
    elapsed_ms: elapsed,
  };
}

export async function performRegexSearch(
  pattern: string,
  viewLines: { viewId: string; lines: string[] }[],
  caseSensitive: boolean,
  limit: number = 100
): Promise<SearchResult> {
  const startTime = performance.now();
  const engine = await getSearchEngine();

  const allLines: { viewId: string; lineIndex: number; text: string }[] = [];
  viewLines.forEach(({ viewId, lines }) => {
    lines.forEach((line, index) => {
      allLines.push({ viewId, lineIndex: index, text: line });
    });
  });

  const rawResults = engine.search_regex(
    pattern,
    allLines.map(l => l.text),
    caseSensitive,
    limit
  );

  const matches: MatchResult[] = rawResults.map((r: any) => ({
    line_index: allLines[r.line_index].lineIndex,
    score: r.score,
    highlights: r.highlights,
    view_id: allLines[r.line_index].viewId,
  }));

  const elapsed = performance.now() - startTime;

  return {
    matches,
    total_count: matches.length,
    query: pattern,
    elapsed_ms: elapsed,
  };
}

export async function performPlainSearch(
  query: string,
  viewLines: { viewId: string; lines: string[] }[],
  caseSensitive: boolean,
  limit: number = 100
): Promise<SearchResult> {
  const startTime = performance.now();
  const engine = await getSearchEngine();

  const allLines: { viewId: string; lineIndex: number; text: string }[] = [];
  viewLines.forEach(({ viewId, lines }) => {
    lines.forEach((line, index) => {
      allLines.push({ viewId, lineIndex: index, text: line });
    });
  });

  const rawResults = engine.search_plain(
    query,
    allLines.map(l => l.text),
    caseSensitive,
    limit
  );

  const matches: MatchResult[] = rawResults.map((r: any) => ({
    line_index: allLines[r.line_index].lineIndex,
    score: r.score,
    highlights: r.highlights,
    view_id: allLines[r.line_index].viewId,
  }));

  const elapsed = performance.now() - startTime;

  return {
    matches,
    total_count: matches.length,
    query,
    elapsed_ms: elapsed,
  };
}

export async function performSearch(
  query: string,
  mode: 'fuzzy' | 'plain' | 'regex',
  viewLines: { viewId: string; lines: string[] }[],
  caseSensitive: boolean,
  limit: number = 100
): Promise<SearchResult> {
  switch (mode) {
    case 'fuzzy':
      return performFuzzySearch(query, viewLines, caseSensitive, limit);
    case 'regex':
      return performRegexSearch(query, viewLines, caseSensitive, limit);
    case 'plain':
      return performPlainSearch(query, viewLines, caseSensitive, limit);
    default:
      return performFuzzySearch(query, viewLines, caseSensitive, limit);
  }
}
