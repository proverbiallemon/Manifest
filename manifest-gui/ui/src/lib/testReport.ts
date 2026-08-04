import type { Report, ReportMod } from "../types";

export function mkMod(over: Partial<ReportMod> & { name: string }): ReportMod {
  return {
    path: `/mods/${over.name}.otr`,
    enabled: true,
    asset_count: 1,
    error: null,
    pinned: null,
    ...over,
  };
}

export function mkReport(over: Partial<Report>): Report {
  return {
    schema_version: 3,
    mods: [],
    conflicts: [],
    warnings: [],
    current_order: [],
    proposed_order: [],
    moves: [],
    ...over,
  };
}
