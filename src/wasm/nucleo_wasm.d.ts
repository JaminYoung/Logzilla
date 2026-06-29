declare module '*.wasm' {
  const content: WebAssembly.Module;
  export default content;
}

declare module '../wasm/nucleo_wasm' {
  export class NucleoSearch {
    constructor();
    search(query: string, lines: string[], case_sensitive: boolean, limit: number): any[];
    search_regex(pattern: string, lines: string[], case_sensitive: boolean, limit: number): any[];
    search_plain(query: string, lines: string[], case_sensitive: boolean, limit: number): any[];
  }
  
  export default function init(): Promise<void>;
}
