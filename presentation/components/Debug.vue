<template>
  <div>
    Debug: Page is {{ $page }} and Current is {{ currentPage }}
    <VueTyping :text="test" :cursorOptions="myCursor" />
  </div>
</template>
