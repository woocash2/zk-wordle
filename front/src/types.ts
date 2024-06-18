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
