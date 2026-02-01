<script setup>
import VueTyping from "vue-typing";

// Define props so you can customize the text from your Markdown
const props = defineProps({
  targetText: {
    type: String,
    default: "Hello World",
  },
});

// Options for the vue-typing library
const myCursor = {
  blinking: true,
  cursor: "|",
  color: "currentColor", // This allows it to inherit your text color
  framerate: 24,
};
</script>

<template>
  <div>
    <div class="terminal-wrapper font-mono text-3xl my-4">
      <VueTyping :text="targetText" :cursorOptions="myCursor" />
    </div>
  </div>
</template>

<style scoped>
/* Optional: ensures the terminal font looks crisp */
.terminal-wrapper {
  letter-spacing: 0.05em;
  min-height: 1.5em; /* Prevents layout jump when text starts typing */
}
</style>
