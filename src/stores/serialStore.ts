import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

interface SerialPortInfo {
  port_name: string;
  description: string;
  vid?: number;
  pid?: number;
}

interface SerialConnection {
  port: string;
  baudrate: number;
  state: "disconnected" | "connecting" | "connected" | "error";
  logs: string[];
}

interface SerialState {
  ports: SerialPortInfo[];
  connectedPorts: Map<string, SerialConnection>;
  selectedPort: string | null;
  baudRate: number;
  isLoading: boolean;
  error: string | null;

  listPorts: () => Promise<void>;
  connect: (port: string, baudrate: number) => Promise<void>;
  disconnect: (port: string) => Promise<void>;
  sendData: (port: string, data: Uint8Array) => Promise<void>;
  appendLog: (port: string, log: string) => void;
  setSelectedPort: (port: string | null) => void;
  setBaudRate: (rate: number) => void;
}

export const useSerialStore = create<SerialState>((set) => ({
  ports: [],
  connectedPorts: new Map(),
  selectedPort: null,
  baudRate: 115200,
  isLoading: false,
  error: null,

  listPorts: async () => {
    set({ isLoading: true, error: null });
    try {
      const ports = await invoke<SerialPortInfo[]>("list_serial_ports");
      set({ ports, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  connect: async (port: string, baudrate: number) => {
    set({ isLoading: true, error: null });
    try {
      await invoke("open_serial_port", { port, baudrate });
      const connection: SerialConnection = {
        port,
        baudrate,
        state: "connected",
        logs: [],
      };
      set((state) => {
        const newMap = new Map(state.connectedPorts);
        newMap.set(port, connection);
        return {
          connectedPorts: newMap,
          selectedPort: port,
          isLoading: false,
        };
      });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  disconnect: async (port: string) => {
    try {
      await invoke("close_serial_port", { port });
      set((state) => {
        const newMap = new Map(state.connectedPorts);
        newMap.delete(port);
        return { connectedPorts: newMap };
      });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  sendData: async (port: string, data: Uint8Array) => {
    try {
      await invoke("send_serial_data", { port, data: Array.from(data) });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  appendLog: (port: string, log: string) => {
    set((state) => {
      const newMap = new Map(state.connectedPorts);
      const conn = newMap.get(port);
      if (conn) {
        newMap.set(port, {
          ...conn,
          logs: [...conn.logs, log],
        });
      }
      return { connectedPorts: newMap };
    });
  },

  setSelectedPort: (port: string | null) => {
    set({ selectedPort: port });
  },

  setBaudRate: (rate: number) => {
    set({ baudRate: rate });
  },
}));
