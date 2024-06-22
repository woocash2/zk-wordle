import "@expo/browser-polyfill";
import {
  Color,
  type Clue,
  type ClueResponse,
  type Commitment,
  type Proof,
  type SerializedProof,
  type StartResponse,
} from "./types";
import { groth16, type Groth16Proof } from "snarkjs";
import { vk_clue, vk_membership } from "./keys";

const ADDRESS = "http://localhost:4000";

function sanitizeProof(proof: SerializedProof): Proof {
  return {
    a: proof.a.split("(")[1].split(")")[0].split(", "),
    b: proof.b
      .split("(")[2]
      .split(" *")[0]
      .split(" + ")
      .concat(proof.b.split("(")[3].split(" *")[0].split(" + ")),
    c: proof.c.split("(")[1].split(")")[0].split(", "),
  };
}

export async function getClue(
  guess: string,
  word_id: string
): Promise<Clue | null> {
  const res = await fetch(`${ADDRESS}/guess`, {
    method: "POST",
    body: JSON.stringify({ guess: guess.toLowerCase(), word_id }),
    headers: {
      "Content-Type": "application/json",
    },
  });
  if (!res.ok) {
    return null;
  }
  const { colors, proof } = (await res.json()) as ClueResponse;

  const sanitizedProof = sanitizeProof(proof);

  console.log(sanitizedProof);
  return {
    clue: colors,
    colors: colors.map((x) =>
      x === 2 ? Color.GREEN : x === 1 ? Color.YELLOW : Color.DARK_GREY
    ),
    proof: sanitizedProof,
  };
}

function getGroth16Proof(proof: Proof): Groth16Proof {
  return {
    pi_a: proof.a,
    pi_b: [
      [proof.b[0], proof.b[1]],
      [proof.b[2], proof.b[3]],
    ],
    pi_c: proof.c,
    curve: "bn128",
    protocol: "groth16",
  };
}

export async function verifyClue(
  guess: string,
  clue: Clue,
  commitment: string
): Promise<boolean> {
  const signals = [
    ...clue.clue.map((x) => x.toString()),
    (guess.charCodeAt(0) - 65).toString(),
    (guess.charCodeAt(1) - 65).toString(),
    (guess.charCodeAt(2) - 65).toString(),
    (guess.charCodeAt(3) - 65).toString(),
    (guess.charCodeAt(4) - 65).toString(),
    commitment,
  ];

  console.log(signals);

  return await groth16.verify(
    vk_clue,
    signals,
    getGroth16Proof(clue.proof),
    console
  );
}

export async function getCommitment(): Promise<Commitment> {
  const res = await fetch(`${ADDRESS}/start`);
  const { commitment, proof, word_id } = (await res.json()) as StartResponse;

  const sanitizedProof = sanitizeProof(proof);

  console.log(sanitizedProof);
  return {
    commitment,
    proof: sanitizedProof,
    word_id: word_id,
  };
}

const rootHash =
  "4768437044799254023802168680693360623505449298048650929961070166353749090917";

export async function verifyCommitment(
  commitment: Commitment
): Promise<boolean> {
  const signals = [rootHash, commitment.commitment];

  console.log(signals);

  return await groth16.verify(
    vk_membership,
    signals,
    getGroth16Proof(commitment.proof),
    console
  );
}
