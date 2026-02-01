<template>
  <div class="ml-0! mt-1.5 text-xs">
    <sup v-if="number" class="mr-1">{{ number }}</sup>
    <slot />
  </div>
</template>

<script setup lang="ts">
defineProps<{
  number?: number;
}>();
</script>
