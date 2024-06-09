import { Color, type Clue } from "./types";

function getLetterCounts(word: string): Map<string, number> {
  let counts = new Map<string, number>();
  for (let letter of word) {
    counts.set(letter, (counts.get(letter) || 0) + 1);
  }
  return counts;
}

const words: string[] = ["HELLO", "WORLD", "APPLE"];
const word = words[Math.floor(Math.random() * words.length)];

export async function getClue(guess: string): Promise<Clue> {
  await new Promise((resolve) => setTimeout(resolve, 1000));
  let greenIndices: number[] = [];
  let yellowIndices: number[] = [];
  const letterCounts = getLetterCounts(word);
  for (let i = 0; i < guess.length; i++) {
    if (guess[i] === word[i]) {
      greenIndices.push(i);
      letterCounts.set(guess[i], letterCounts.get(guess[i])! - 1);
    }
  }

  for (let i = 0; i < guess.length; i++) {
    if ((letterCounts.get(guess[i]) || 0) > 0 && guess[i] !== word[i]) {
      yellowIndices.push(i);
      letterCounts.set(guess[i], letterCounts.get(guess[i])! - 1);
    }
  }

  let colors = Array(5).fill(Color.DARK_GREY);
  greenIndices.forEach((i) => (colors[i] = Color.GREEN));
  yellowIndices.forEach((i) => (colors[i] = Color.YELLOW));
  return {
    colors,
    proof: Math.random(),
  };
}

export async function verifyClue(clue: Clue): Promise<boolean> {
  await new Promise((resolve) => setTimeout(resolve, 1000));
  return true;
}
