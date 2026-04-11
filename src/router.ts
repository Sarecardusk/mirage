import { createRouter, createWebHistory } from "vue-router";
import ChatView from "@/views/ChatView.vue";
import SettingsView from "@/views/SettingsView.vue";
import ThemeCardListView from "@/views/ThemeCardListView.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "theme-card-list",
      component: ThemeCardListView,
    },
    {
      path: "/chat/:themeCardId",
      name: "chat",
      component: ChatView,
      props: true,
    },
    {
      path: "/settings",
      name: "settings",
      component: SettingsView,
    },
  ],
});

export default router;
