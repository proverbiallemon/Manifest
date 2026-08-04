export interface ReportMod {
  name: string;
  path: string;
  enabled: boolean;
  asset_count: number;
  error: string | null;
  pinned: "top" | "bottom" | null;
}

export interface ReportConflict {
  asset: string;
  providers: string[];
  winner: string;
}

export type Warning =
  | { kind: "total_eclipse"; name: string }
  | { kind: "mutual_overlap"; names: string[] }
  | { kind: "unlistable"; name: string; reason: string }
  | { kind: "duplicate_gamebanana_mod"; mod_id: number; names: string[] };

export interface Move {
  name: string;
  reason: string;
}

export interface Report {
  schema_version: number;
  mods: ReportMod[];
  conflicts: ReportConflict[];
  warnings: Warning[];
  current_order: string[];
  proposed_order: string[];
  moves: Move[];
}

export interface StoredSettings {
  config_path: string;
  mods_dir: string;
}

export function conflictCount(report: Report, modName: string): number {
  return report.conflicts.filter((c) => c.providers.includes(modName)).length;
}

export function sortNeeded(report: Report): boolean {
  return (
    JSON.stringify(report.current_order) !== JSON.stringify(report.proposed_order)
  );
}

export interface OverlapSummary {
  prevailsOver: { name: string; count: number }[];
  overriddenBy: { name: string; count: number }[];
  contestedAssets: string[];
}

export function overlapsFor(report: Report, modName: string): OverlapSummary {
  const prevails = new Map<string, number>();
  const overridden = new Map<string, number>();
  const assets: string[] = [];
  for (const c of report.conflicts) {
    if (!c.providers.includes(modName)) continue;
    assets.push(c.asset);
    if (c.winner === modName) {
      for (const p of c.providers) {
        if (p !== modName) prevails.set(p, (prevails.get(p) ?? 0) + 1);
      }
    } else {
      overridden.set(c.winner, (overridden.get(c.winner) ?? 0) + 1);
    }
  }
  const toSorted = (m: Map<string, number>) =>
    [...m.entries()]
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count || a.name.localeCompare(b.name));
  return {
    prevailsOver: toSorted(prevails),
    overriddenBy: toSorted(overridden),
    contestedAssets: assets,
  };
}
