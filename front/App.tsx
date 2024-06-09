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
import { Color, type Guess } from "./src/types";
import { getClue, verifyClue } from "./src/verify";
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

function InputRow({ text, isLoading }: { text: string; isLoading: boolean }) {
  const sv = useSharedValue(1);

  useEffect(() => {
    sv.value = isLoading
      ? withRepeat(withTiming(0.5, { duration: 600 }), -1, true)
      : 1;
  }, [isLoading]);

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
          style={styles.cell}
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
    if (text.length === 5) {
      setIsLoading(true);
      getClue(text).then((clue) => {
        verifyClue(clue).then((valid) => {
          if (!valid) {
          }
          setGuesses([...guesses, { word: text, colors: clue.colors }]);
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
          <InputRow key={guesses.length} text={text} isLoading={isLoading} />
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
});
