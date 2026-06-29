import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

interface DcfHeader {
  magic: number[];
  data_size: number;
  timestamp: number;
  company: number[];
  version: number;
  config_tree_offset: number;
  info_offset: number;
  info_version: number;
  info_len: number;
  code_version: number;
}

interface ConfigOption {
  label: string;
  value: number;
}

type ConfigValue =
  | { Bool: boolean }
  | { Int: number }
  | { String: string }
  | { Mac: string };

interface ConfigItem {
  entry_type: string;
  label_cn: string;
  name: string;
  tooltip: string;
  var_name: string;
  offset: number;
  bit_offset: number;
  bit_width: number;
  size: number;
  value: ConfigValue | null;
  default_value: ConfigValue | null;
  min_val: number;
  max_val: number;
  options: ConfigOption[];
  val_type: number;
  addr_start: number[] | null;
  addr_end: number[] | null;
  addr_set: number[] | null;
  str_length: number;
  children: ConfigItem[];
  level_value: number;
  ui_condition_var: string | null;
}

interface DcfInfo {
  path: string;
  header: DcfHeader;
  config_tree: ConfigItem[];
  info_offset: number;
  info_len: number;
  info_data: number[];
}

interface DcfState {
  dcfInfo: DcfInfo | null;
  dcfData: number[] | null;
  isLoading: boolean;
  error: string | null;

  loadDcf: (path: string) => Promise<void>;
  saveDcf: (path: string) => Promise<void>;
  clearDcf: () => void;
}

export const useDcfStore = create<DcfState>((set, get) => ({
  dcfInfo: null,
  dcfData: null,
  isLoading: false,
  error: null,

  loadDcf: async (path: string) => {
    set({ isLoading: true, error: null });
    try {
      const info = await invoke<DcfInfo>("open_dcf", { path });
      set({
        dcfInfo: info,
        dcfData: null, // Will be loaded separately if needed
        isLoading: false,
      });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  saveDcf: async (path: string) => {
    const { dcfData } = get();
    if (!dcfData) {
      set({ error: "No DCF data to save" });
      return;
    }
    try {
      await invoke("save_dcf", { path, dcfData });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  clearDcf: () => {
    set({ dcfInfo: null, dcfData: null, error: null });
  },
}));
