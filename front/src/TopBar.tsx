import React from "react";
import { StyleSheet, View } from "react-native";
import { BigKey } from "./Keyboard";

export function TopBar({ onReset }: { onReset: () => void }) {
  return (
    <View style={styles.topBar}>
      <BigKey char="↺" onPress={onReset} />
    </View>
  );
}

const styles = StyleSheet.create({
  topBar: {
    flexDirection: "row",
    justifyContent: "flex-end",
    width: "100%",
    padding: 10,
  },
});
