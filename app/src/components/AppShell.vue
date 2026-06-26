<template>
  <main class="app-shell">
    <div class="window-chrome">
      <slot name="title" />
    </div>
    <slot name="tabs" />
    <slot />
    <slot name="status" />
  </main>
</template>
