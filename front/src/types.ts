export enum Color {
  GREY = "#808080",
  DARK_GREY = "#404040",
  GREEN = "#118811",
  YELLOW = "#DDDD00",
}

export type Proof = number;

export type Clue = {
  colors: Color[];
  proof: Proof;
};

export type Guess = {
  word: string;
  colors: Color[];
};
