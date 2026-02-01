<template>
  <footer
    class="absolute bottom-5 right-5 p-2.1 text-right text-xs opacity-50"
    v-if="$nav.currentPage !== 1"
  >
    Nick Rieser – # <SlideCurrentNo /> of <SlidesTotal />
  </footer>
</template>
