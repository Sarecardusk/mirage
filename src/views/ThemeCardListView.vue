<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import AppLayout from "@/components/AppLayout.vue";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { createThemeCard, listThemeCards } from "@/services/themeCard";
import type { ThemeCard } from "@/types/bindings";

const router = useRouter();

const cards = ref<ThemeCard[]>([]);
const loading = ref(false);
const creating = ref(false);
const createError = ref("");
const loadError = ref("");
const dialogOpen = ref(false);

const formName = ref("");
const formSystemPrompt = ref("");

async function refreshCards() {
  loading.value = true;
  loadError.value = "";
  try {
    cards.value = await listThemeCards();
  } catch (error) {
    loadError.value = error instanceof Error ? error.message : String(error);
  } finally {
    loading.value = false;
  }
}

async function handleCreateThemeCard() {
  creating.value = true;
  createError.value = "";
  try {
    const card = await createThemeCard({
      name: formName.value,
      systemPrompt: formSystemPrompt.value,
    });
    formName.value = "";
    formSystemPrompt.value = "";
    dialogOpen.value = false;
    await refreshCards();
    await router.push(`/chat/${card.id}`);
  } catch (error) {
    createError.value = error instanceof Error ? error.message : String(error);
  } finally {
    creating.value = false;
  }
}

function openThemeCard(card: ThemeCard) {
  router.push(`/chat/${card.id}`);
}

onMounted(() => {
  refreshCards();
});
</script>

<template>
  <AppLayout>
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-lg font-semibold">Theme Cards</h3>
          <p class="text-sm text-muted-foreground">创建一张卡片并开始聊天。</p>
        </div>
        <Dialog v-model:open="dialogOpen">
          <DialogTrigger as-child>
            <Button>新建 Theme Card</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>新建 Theme Card</DialogTitle>
              <DialogDescription>输入名称和系统提示词，马上开始对话。</DialogDescription>
            </DialogHeader>
            <div class="space-y-3">
              <div class="space-y-1">
                <label class="text-sm font-medium">名称</label>
                <Input v-model="formName" placeholder="例如：赛博侦探" />
              </div>
              <div class="space-y-1">
                <label class="text-sm font-medium">系统提示词</label>
                <Textarea v-model="formSystemPrompt" placeholder="描述角色设定、说话风格与边界。" />
              </div>
              <p v-if="createError" class="text-sm text-destructive">{{ createError }}</p>
            </div>
            <DialogFooter>
              <Button :disabled="creating" @click="handleCreateThemeCard">
                {{ creating ? "创建中..." : "创建并聊天" }}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      <p v-if="loadError" class="text-sm text-destructive">{{ loadError }}</p>

      <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        <Card
          v-for="card in cards"
          :key="card.id"
          class="cursor-pointer transition-colors hover:bg-accent/40"
          @click="openThemeCard(card)"
        >
          <CardHeader>
            <CardTitle class="text-base">{{ card.name }}</CardTitle>
          </CardHeader>
          <CardContent class="text-xs text-muted-foreground">
            创建于 {{ new Date(card.createdAt).toLocaleString() }}
          </CardContent>
        </Card>
      </div>

      <div v-if="loading" class="text-sm text-muted-foreground">加载中...</div>
      <div v-else-if="cards.length === 0" class="rounded-lg border border-dashed p-6 text-sm">
        还没有 Theme Card，先创建第一张吧。
      </div>
    </div>
  </AppLayout>
</template>
