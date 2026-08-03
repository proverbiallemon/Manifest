export interface ReportMod {
  name: string;
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
