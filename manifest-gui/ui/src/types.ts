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

export interface ModToggle {
  path: string;
  enabled: boolean;
}

export function conflictCount(report: Report, modName: string): number {
  return report.conflicts.filter((c) => c.providers.includes(modName)).length;
}

export function sortNeeded(report: Report): boolean {
  // Names of disabled or removed mods linger in EnabledMods until the game's
  // next boot rewrites it; they say nothing about ordering, so compare only
  // the names that can actually load.
  const loadable = new Set(
    report.mods.filter((m) => m.enabled).map((m) => m.name)
  );
  const current = report.current_order.filter((n) => loadable.has(n));
  const proposed = report.proposed_order.filter((n) => loadable.has(n));
  return JSON.stringify(current) !== JSON.stringify(proposed);
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

export interface Zones {
  fore: ReportMod[];
  free: ReportMod[];
  aft: ReportMod[];
}

// Loaded mods (enabled and present in EnabledMods) grouped into physical
// zones. Display order within each zone follows current_order; duplicate
// names keep one row per file.
export function deriveZones(report: Report): Zones {
  const orderIndex = new Map(report.current_order.map((n, i) => [n, i]));
  const loaded = report.mods
    .filter((m) => m.enabled && orderIndex.has(m.name))
    .slice()
    .sort(
      (a, b) =>
        orderIndex.get(a.name)! - orderIndex.get(b.name)! ||
        a.path.localeCompare(b.path)
    );
  return {
    fore: loaded.filter((m) => m.pinned === "top"),
    free: loaded.filter((m) => m.pinned === null),
    aft: loaded.filter((m) => m.pinned === "bottom"),
  };
}

export function zoneNameLists(z: Zones): {
  fore: string[];
  free: string[];
  aft: string[];
} {
  const dedupe = (mods: ReportMod[]) => [...new Set(mods.map((m) => m.name))];
  return { fore: dedupe(z.fore), free: dedupe(z.free), aft: dedupe(z.aft) };
}

// For a zone's rows (which may repeat a name across duplicate files), the row
// index of each unique name's first occurrence, in the same first-seen order
// zoneNameLists produces. Lets drag code pick one representative row element
// per deduped name.
export function firstRowIndexes(mods: ReportMod[]): number[] {
  const seen = new Set<string>();
  const out: number[] = [];
  mods.forEach((m, i) => {
    if (!seen.has(m.name)) {
      seen.add(m.name);
      out.push(i);
    }
  });
  return out;
}

// True when the actual EnabledMods sequence no longer equals fore+free+aft,
// which happens when the game's boot rewrite moves things behind our back.
export function zonesShifted(report: Report): boolean {
  const names = zoneNameLists(deriveZones(report));
  const zoneSeq = [...names.fore, ...names.free, ...names.aft];
  const loadable = new Set(zoneSeq);
  const actual = report.current_order.filter((n) => loadable.has(n));
  return JSON.stringify(actual) !== JSON.stringify(zoneSeq);
}
