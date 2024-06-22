export enum Color {
  GREY = "#808080",
  DARK_GREY = "#404040",
  GREEN = "#118811",
  YELLOW = "#DDDD00",
}

export type Proof = {
  a: string[];
  b: string[];
  c: string[];
};

export type Clue = {
  clue: number[];
  colors: Color[];
  proof: Proof;
};

export type Guess = {
  word: string;
  colors: Color[];
};

export type SerializedProof = {
  a: string;
  b: string;
  c: string;
};

export type ClueResponse = {
  colors: number[];
  proof: SerializedProof;
};

export type StartResponse = {
  commitment: string;
  proof: SerializedProof;
  word_id: string;
};

export type Commitment = {
  commitment: string;
  proof: Proof;
  word_id: string;
};

export type Result<T> =
  | { type: "ok"; value: T }
  | { type: "error"; error: string };
