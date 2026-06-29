import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

interface FlashProgress {
  state: string;
  percent: number;
  message: string;
  current_op: string;
}

interface FlashState {
  isFlashing: boolean;
  progress: FlashProgress;
  error: string | null;

  startFlash: (dcfData: number[], port: string) => Promise<void>;
  cancelFlash: () => Promise<void>;
  getProgress: () => Promise<void>;
}

export const useFlashStore = create<FlashState>((set, get) => ({
  isFlashing: false,
  progress: {
    state: "idle",
    percent: 0,
    message: "",
    current_op: "",
  },
  error: null,

  startFlash: async (dcfData: number[], port: string) => {
    set({ isFlashing: true, error: null });
    try {
      await invoke("start_flash", { dcfData, port });
      // Start polling for progress
      const pollProgress = async () => {
        const { isFlashing } = get();
        if (!isFlashing) return;

        try {
          const progress = await invoke<FlashProgress>("get_flash_progress");
          set({ progress });

          if (progress.state === "done" || progress.state.startsWith("error")) {
            set({ isFlashing: false });
            return;
          }

          setTimeout(pollProgress, 100);
        } catch (error) {
          set({ error: String(error), isFlashing: false });
        }
      };
      pollProgress();
    } catch (error) {
      set({ error: String(error), isFlashing: false });
    }
  },

  cancelFlash: async () => {
    try {
      await invoke("cancel_flash");
      set({ isFlashing: false });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  getProgress: async () => {
    try {
      const progress = await invoke<FlashProgress>("get_flash_progress");
      set({ progress });
    } catch (error) {
      set({ error: String(error) });
    }
  },
}));
