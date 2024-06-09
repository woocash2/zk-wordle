import React from "react";
import { Text, StyleSheet } from "react-native";
import Animated, { ZoomInEasyDown } from "react-native-reanimated";

export function ResultView({ won }: { won: boolean }) {
  return (
    <Animated.View
      entering={ZoomInEasyDown.springify().delay(800)}
      style={styles.modal}
    >
      <Text style={styles.text}>You {won ? "won" : "lost"}</Text>
    </Animated.View>
  );
}

const styles = StyleSheet.create({
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
});
