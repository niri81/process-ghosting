<template>
  <div
    class="absolute bottom-5 left-5 w-80% opacity-50"
    v-bind:class="{
      'bg-main': filled,
      'z-1': filled,
    }"
  >
    <hr v-if="separator" />
    <ul
      class="flex flex-wrap !list-none px-3 py-2"
      v-bind:class="{
        'flex-col': y === 'col',
        'items-start': x === 'l' && y === 'col',
        'items-end': x === 'r' && y === 'col',
        'justify-start': x === 'l' && y === 'row',
        'justify-end': x === 'r' && y === 'row',
        'gap-col-3': y === 'row',
        'gap-row-0.5': y === 'col',
      }"
    >
      <slot />
    </ul>
  </div>
</template>

<script setup lang="ts">
import { PropType } from "vue";

defineProps({
  filled: {
    default: false,
    type: Boolean,
  },
  separator: {
    default: false,
    type: Boolean,
  },
  x: {
    default: "l",
    type: String as PropType<"l" | "r">,
    validator: (value) => value === "l" || value === "r",
  },
  y: {
    default: "row",
    type: String as PropType<"col" | "row">,
    validator: (value) => value === "col" || value === "row",
  },
});
</script>
