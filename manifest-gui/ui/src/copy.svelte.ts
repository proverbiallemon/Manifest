import { appState } from "./state.svelte";

export type Voice = "ship" | "plain";

const STRINGS = {
  chooseConfig: { ship: "config", plain: "config file" },
  chooseHold: { ship: "hold", plain: "mods folder" },
  rescan: { ship: "take inventory", plain: "rescan" },
  restow: { ship: "Re-stow the hold", plain: "Sort load order" },
  inOrder: { ship: "hold is in order", plain: "load order is optimal" },
  settings: { ship: "settings", plain: "settings" },
  ledgerHeading: { ship: "LADING, IN ORDER OF LOAD", plain: "LOAD ORDER" },
  notLoaded: { ship: "NOT LOADED", plain: "NOT LOADED" },
  clearMod: { ship: "clear: no contested cargo", plain: "no conflicts" },
  clearStatus: { ship: "clear", plain: "clear" },
  prevailsOver: { ship: "prevails over", plain: "overrides" },
  overriddenBy: { ship: "overridden by", plain: "overridden by" },
  entrySingular: { ship: "entry", plain: "asset" },
  entryPlural: { ship: "entries", plain: "assets" },
  selectEntry: { ship: "select an entry", plain: "select a mod" },
  unlistable: { ship: "could not be inventoried", plain: "could not be read" },
  modalTitle: { ship: "RE-STOW THE HOLD", plain: "SORT LOAD ORDER" },
  colBefore: { ship: "AS STOWED", plain: "CURRENT" },
  colAfter: { ship: "AS PROPOSED", plain: "PROPOSED" },
  confirm: { ship: "Stamp it", plain: "Apply" },
  cancel: { ship: "Belay that", plain: "Cancel" },
  modalNote: {
    ship: "entries move; nothing is written until stamped",
    plain: "mods move; nothing is written until applied",
  },
  damagedTitle: { ship: "DAMAGED MANIFEST", plain: "SOMETHING WENT WRONG" },
  nothingChanged: { ship: "nothing was changed", plain: "nothing was changed" },
  locateConfig: { ship: "locate the manifest", plain: "choose config file" },
  locateHold: { ship: "locate the hold", plain: "choose mods folder" },
  tryAgain: { ship: "try again", plain: "try again" },
  loading: { ship: "taking inventory of the hold...", plain: "scanning mods..." },
  noConfig: { ship: "no manifest on file", plain: "no config file found" },
  eclipseHeadline: { ship: "is fully covered by later cargo", plain: "is fully overridden by later mods" },
  eclipseGuidance: { ship: "it does nothing where it sits; consider unloading it", plain: "it has no effect; consider disabling it" },
  overlapHeadline: { ship: "carry identical cargo", plain: "contain identical assets" },
  overlapGuidance: { ship: "only the last one prevails; consider keeping one", plain: "only the last one takes effect; consider keeping one" },
  unlistableHeadline: { ship: "could not be inventoried", plain: "could not be read" },
  duplicateHeadline: { ship: "are the same shipment twice", plain: "are the same mod installed twice" },
  duplicateGuidance: { ship: "consider removing the extra copy", plain: "consider removing the extra copy" },
} as const;

export type CopyKey = keyof typeof STRINGS;

export function t(key: CopyKey): string {
  return STRINGS[key][appState.voice];
}
