// Hand-written for the spike; to be generated from the Rust schema (ts-rs/specta).
export type Align = "start" | "end" | "center" | "stretch";

export interface NodeStyle {
  columns?: string;
  rows?: string;
  areas?: string[];
  autoRows?: string;
  autoColumns?: string;
  gap?: number;
  rowGap?: number;
  columnGap?: number;
  padding?: number;
  area?: string;
  column?: string;
  row?: string;
  minWidth?: number;
  minHeight?: number;
  justifySelf?: Align;
  alignSelf?: Align;
}

export interface Node {
  id: string;
  kind?: "grid" | "box";
  style?: NodeStyle;
  fill?: string;
  children?: Node[];
}

export interface Document {
  width: number;
  height: number;
  root: Node;
}

export interface Placed {
  id: string;
  depth: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

export function initLayout(): Promise<unknown>;
export function layoutDocument(doc: Document): Placed[];
