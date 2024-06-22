import React, { useEffect } from "react";
import { Text, View, StyleSheet } from "react-native";
import Animated, {
  useAnimatedStyle,
  useSharedValue,
  withTiming,
} from "react-native-reanimated";
import { Color } from "./types";

const KEYBOARD_ROWS: string[][] = [
  ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
  ["A", "S", "D", "F", "G", "H", "J", "K", "L"],
  ["Z", "X", "C", "V", "B", "N", "M"],
];

export function BigKey({
  char,
  onPress,
  isLoading,
}: {
  char: string;
  onPress: () => void;
  isLoading?: boolean;
}) {
  const sv = useSharedValue<string>(Color.DARK_GREY);

  useEffect(() => {
    sv.value = withTiming(isLoading ? Color.DARK_GREY : Color.GREY);
  }, [isLoading]);

  const animatedStyle = useAnimatedStyle(() => {
    return {
      backgroundColor: sv.value,
    };
  });

  return (
    <Animated.Text
      selectable={false}
      onPress={onPress}
      style={[styles.bigKey, animatedStyle]}
    >
      {char}
    </Animated.Text>
  );
}

export function Keyboard({
  onPress,
  onSubmit,
  onBack,
  greenLetters,
  yellowLetters,
  darkGreyLetters,
  isLoading,
}: {
  onPress: (char: string) => void;
  onSubmit: () => void;
  onBack: () => void;
  greenLetters: Set<string>;
  yellowLetters: Set<string>;
  darkGreyLetters: Set<string>;
  isLoading: boolean;
}) {
  return (
    <View style={styles.keyboardContainer}>
      {KEYBOARD_ROWS.map((row, i) => (
        <Animated.View key={i} style={styles.row}>
          {i === 2 && (
            <BigKey char="⏎" isLoading={isLoading} onPress={onSubmit} />
          )}
          {row.map((char, j) => (
            <Text
              selectable={false}
              onPress={() => onPress(char)}
              key={j}
              style={[
                styles.key,
                {
                  backgroundColor: greenLetters.has(char)
                    ? Color.GREEN
                    : yellowLetters.has(char)
                    ? Color.YELLOW
                    : darkGreyLetters.has(char)
                    ? Color.DARK_GREY
                    : Color.GREY,
                },
              ]}
            >
              {char}
            </Text>
          ))}
          {i === 2 && <BigKey char="⌫" onPress={onBack} />}
        </Animated.View>
      ))}
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
  key: {
    margin: 2,
    fontSize: 32,
    fontWeight: "500",
    width: 30,
    height: 50,
    textAlign: "center",
    textAlignVertical: "center",
    borderRadius: 3,
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
  modal: {
    top: "50%",
    position: "absolute",
    flex: 1,
    justifyContent: "center",
    alignItems: "center",
    backgroundColor: "#333333",
    borderColor: "black",
    borderStyle: "solid",
    borderWidth: 2,
    borderRadius: 10,
    width: 300,
    height: 120,
    zIndex: 1000,
  },
  keyboardContainer: {
    flex: 1,
    justifyContent: "flex-end",
    flexDirection: "column",
    alignItems: "center",
  },
  bigKey: {
    margin: 2,
    fontSize: 32,
    fontWeight: "500",
    width: 60,
    height: 50,
    textAlign: "center",
    textAlignVertical: "center",
    borderRadius: 3,
    backgroundColor: Color.GREY,
  },
  topBar: {
    flexDirection: "row",
    justifyContent: "flex-end",
    width: "100%",
    padding: 10,
  },
});
