<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";

const route = useRoute();

const navItems = [
  { to: "/", label: "Theme Cards" },
  { to: "/settings", label: "Settings" },
];

function isActive(path: string): boolean {
  if (path === "/") {
    return route.path === "/";
  }
  return route.path.startsWith(path);
}

const title = computed(() => {
  if (route.path.startsWith("/chat/")) {
    return "Chat";
  }
  if (route.path.startsWith("/settings")) {
    return "Settings";
  }
  return "Theme Cards";
});
</script>

<template>
  <div class="flex min-h-screen">
    <aside class="w-56 border-r bg-muted/30 p-4">
      <h1 class="mb-4 text-lg font-semibold">Mirage MVP</h1>
      <nav class="space-y-2">
        <RouterLink
          v-for="item in navItems"
          :key="item.to"
          :to="item.to"
          class="block rounded px-3 py-2 text-sm"
          :class="isActive(item.to) ? 'bg-primary text-primary-foreground' : 'hover:bg-accent'"
        >
          {{ item.label }}
        </RouterLink>
      </nav>
    </aside>
    <main class="min-w-0 flex-1 p-6">
      <header class="mb-4">
        <h2 class="text-xl font-semibold">{{ title }}</h2>
      </header>
      <slot />
    </main>
  </div>
</template>
