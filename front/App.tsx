import React, { useEffect, useRef, useState } from "react";
import { LayoutAnimation, StyleSheet, Text, View } from "react-native";
import Animated, {
  FadeIn,
  FadeInUp,
  FadeOut,
  FlipInEasyX,
  LayoutAnimationConfig,
  LinearTransition,
  useAnimatedStyle,
  useSharedValue,
  withRepeat,
  withTiming,
} from "react-native-reanimated";
import { TopBar } from "./src/TopBar";
import { Color, type Commitment, type Guess } from "./src/types";
import {
  getClue,
  getCommitment,
  verifyClue,
  verifyCommitment,
} from "./src/api";
import { Keyboard } from "./src/Keyboard";
import { ResultView } from "./src/ResultView";

function Row({ word, colors }: { word: string; colors: Color[] }) {
  return (
    <View style={styles.row}>
      {word.split("").map((char, i) => (
        <Animated.View
          key={i}
          entering={FlipInEasyX.delay(100 * i).springify()}
        >
          <Text style={[styles.cell, { backgroundColor: colors[i] }]}>
            {char}
          </Text>
        </Animated.View>
      ))}
    </View>
  );
}

function InputRow({
  text,
  isLoading,
  isInvalid,
}: {
  text: string;
  isLoading: boolean;
  isInvalid: boolean;
}) {
  const sv = useSharedValue(1);

  useEffect(() => {
    sv.value = isLoading
      ? withRepeat(withTiming(0.5, { duration: 600 }), -1, true)
      : 1;
    if (isInvalid) {
      sv.value = 1;
    }
  }, [isLoading, isInvalid]);

  const animatedStyle = useAnimatedStyle(() => {
    return {
      opacity: sv.value,
    };
  });

  let trimmedText = text.slice(0, 5);
  while (trimmedText.length < 5) {
    trimmedText += " ";
  }

  return (
    <Animated.View
      key={"input"}
      layout={LinearTransition}
      style={[styles.row, animatedStyle]}
    >
      {trimmedText.split("").map((char, i) => (
        <Animated.View
          entering={FadeIn}
          exiting={FadeOut.delay(100 * i)}
          key={i}
          style={[styles.cell, isInvalid && { backgroundColor: "red" }]}
        >
          <LayoutAnimationConfig skipExiting>
            <Animated.Text
              key={i + char}
              entering={FadeInUp}
              exiting={FadeOut}
              style={styles.textInput}
            >
              {char}
            </Animated.Text>
          </LayoutAnimationConfig>
        </Animated.View>
      ))}
    </Animated.View>
  );
}

export default function App() {
  const [guesses, setGuesses] = useState<Guess[]>([]);
  const [text, setText] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const greenLetters = useRef(new Set<string>());
  const yellowLetters = useRef(new Set<string>());
  const darkGreyLetters = useRef(new Set<string>());
  const [commitment, setCommitment] = useState<Commitment | null>(null);
  const [isInvalid, setIsInvalid] = useState<boolean>(false);
  const [isMembershipVerified, setIsMembershipVerified] =
    useState<boolean>(false);

  useEffect(() => {
    getCommitment().then(setCommitment);
  }, []);

  useEffect(() => {
    if (commitment) {
      verifyCommitment(commitment).then((valid) => {
        if (!valid) {
          setIsInvalid(true);
        } else {
          setIsMembershipVerified(true);
        }
      });
    }
  }, [commitment]);

  const updateColors = (guess: Guess) => {
    for (let i = 0; i < guess.colors.length; i++) {
      if (guess.colors[i] === Color.GREEN) {
        greenLetters.current.add(guess.word[i]);
      } else if (guess.colors[i] === Color.YELLOW) {
        yellowLetters.current.add(guess.word[i]);
      } else {
        darkGreyLetters.current.add(guess.word[i]);
      }
    }
  };

  const finished =
    guesses.length === 6 ||
    (guesses.length > 0 &&
      guesses[guesses.length - 1].colors.every(
        (color) => color === Color.GREEN
      ));

  const updateText = (char: string) => {
    if (isLoading) return;
    setText((prev) => (prev.length < 5 ? prev + char : prev));
  };

  const onReset = () => {
    if (isLoading) return;
    setGuesses([]);
    setText("");
    greenLetters.current.clear();
    yellowLetters.current.clear();
    darkGreyLetters.current.clear();
  };

  const onSubmit = () => {
    if (text.length === 5 && commitment !== null) {
      setIsLoading(true);
      getClue(text).then((clue) => {
        verifyClue(text, clue, commitment.commitment).then((valid) => {
          if (!valid) {
            setIsInvalid(true);
            return;
          }
          setGuesses([...guesses, { word: text, colors: clue.colors }]);
          updateColors({ word: text, colors: clue.colors });
          setText("");
          setIsLoading(false);
        });
      });
    }
  };

  const onBack = () => {
    if (isLoading) return;
    setText((prev) => prev.slice(0, prev.length - 1));
  };

  return (
    <View style={styles.container}>
      <TopBar onReset={onReset} />
      <Text style={styles.hash}>
        Current hash: {commitment?.commitment}
        {isMembershipVerified ? " ✅" : " ⏱️"}
      </Text>
      {isInvalid && (
        <Text style={styles.redText}>{"The server is lying to you :(("}</Text>
      )}
      {finished && (
        <ResultView
          won={guesses[guesses.length - 1].colors.every(
            (color) => color === Color.GREEN
          )}
        />
      )}
      <View style={{ padding: 40 }}>
        {guesses.map((guess, i) => (
          <Row key={i} word={guess.word} colors={guess.colors} />
        ))}
        {!finished && (
          <InputRow
            key={guesses.length}
            text={text}
            isLoading={isLoading}
            isInvalid={isInvalid}
          />
        )}
      </View>
      <Keyboard
        greenLetters={greenLetters.current}
        yellowLetters={yellowLetters.current}
        darkGreyLetters={darkGreyLetters.current}
        onPress={updateText}
        onSubmit={onSubmit}
        onBack={onBack}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: "center",
    backgroundColor: "#111111",
    justifyContent: "flex-start",
    padding: 20,
  },
  row: {
    flexDirection: "row",
  },
  cell: {
    margin: 5,
    fontSize: 50,
    fontWeight: "500",
    width: 60,
    height: 60,
    textAlign: "center",
    textAlignVertical: "center",
    borderRadius: 5,
    backgroundColor: Color.GREY,
  },
  textInput: {
    fontSize: 50,
    fontWeight: "500",
    width: 60,
    height: 60,
    textAlign: "center",
    textAlignVertical: "center",
  },
  text: {
    color: "white",
    fontSize: 50,
  },
  hash: {
    color: "white",
    fontSize: 10,
  },
  redText: {
    color: "red",
    fontSize: 50,
  },
});
