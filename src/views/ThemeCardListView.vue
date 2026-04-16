<script setup lang="ts">
import { Pencil, Trash2 } from "lucide-vue-next";
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
import {
  createThemeCard,
  deleteThemeCard,
  listThemeCards,
  updateThemeCard,
} from "@/services/themeCard";
import type { ThemeCard } from "@/types/bindings";

const router = useRouter();

const cards = ref<ThemeCard[]>([]);
const loading = ref(false);
const loadError = ref("");

// ── 新建 ────────────────────────────────────────────────────────────────────
const creating = ref(false);
const createError = ref("");
const createDialogOpen = ref(false);
const formName = ref("");
const formSystemPrompt = ref("");

// ── 编辑 ────────────────────────────────────────────────────────────────────
const editTarget = ref<ThemeCard | null>(null); // null 表示编辑弹窗关闭
const editName = ref("");
const editSystemPrompt = ref("");
const updating = ref(false);
const updateError = ref("");

// ── 删除 ────────────────────────────────────────────────────────────────────
const deleteTarget = ref<ThemeCard | null>(null); // null 表示删除确认弹窗关闭
const deleting = ref(false);
const deleteError = ref("");

// ── 列表刷新 ─────────────────────────────────────────────────────────────────
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

// ── 新建操作 ─────────────────────────────────────────────────────────────────
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
    createDialogOpen.value = false;
    await refreshCards();
    await router.push(`/chat/${card.id}`);
  } catch (error) {
    createError.value = error instanceof Error ? error.message : String(error);
  } finally {
    creating.value = false;
  }
}

// ── 编辑操作 ─────────────────────────────────────────────────────────────────
function openEditDialog(card: ThemeCard) {
  editTarget.value = card;
  editName.value = card.name;
  editSystemPrompt.value = card.systemPrompt;
  updateError.value = "";
}

async function handleUpdateThemeCard() {
  if (!editTarget.value) return;
  updating.value = true;
  updateError.value = "";
  try {
    await updateThemeCard({
      themeCardId: editTarget.value.id,
      name: editName.value,
      systemPrompt: editSystemPrompt.value,
    });
    editTarget.value = null;
    await refreshCards();
  } catch (error) {
    updateError.value = error instanceof Error ? error.message : String(error);
  } finally {
    updating.value = false;
  }
}

// ── 删除操作 ─────────────────────────────────────────────────────────────────
function openDeleteDialog(card: ThemeCard) {
  deleteTarget.value = card;
  deleteError.value = "";
}

async function handleDeleteThemeCard() {
  if (!deleteTarget.value) return;
  deleting.value = true;
  deleteError.value = "";
  try {
    await deleteThemeCard(deleteTarget.value.id);
    deleteTarget.value = null;
    await refreshCards();
  } catch (error) {
    deleteError.value = error instanceof Error ? error.message : String(error);
  } finally {
    deleting.value = false;
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

        <Dialog v-model:open="createDialogOpen">
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
          <CardHeader class="flex-row items-start justify-between space-y-0">
            <CardTitle class="text-base leading-snug">{{ card.name }}</CardTitle>
            <!-- 操作按钮区：阻止冒泡以免触发卡片导航 -->
            <div class="flex shrink-0 gap-1" @click.stop>
              <button
                class="rounded p-1 text-muted-foreground hover:bg-accent hover:text-foreground"
                title="编辑"
                @click="openEditDialog(card)"
              >
                <Pencil :size="14" />
              </button>
              <button
                class="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                title="删除"
                @click="openDeleteDialog(card)"
              >
                <Trash2 :size="14" />
              </button>
            </div>
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

  <!-- 编辑 Dialog（挂在 AppLayout 外，确保层叠顺序不受 card 影响） -->
  <Dialog
    :open="editTarget !== null"
    @update:open="
      (v) => {
        if (!v) editTarget = null;
      }
    "
  >
    <DialogContent>
      <DialogHeader>
        <DialogTitle>编辑 Theme Card</DialogTitle>
        <DialogDescription>修改名称或系统提示词，保存后立即生效。</DialogDescription>
      </DialogHeader>
      <div class="space-y-3">
        <div class="space-y-1">
          <label class="text-sm font-medium">名称</label>
          <Input v-model="editName" placeholder="例如：赛博侦探" />
        </div>
        <div class="space-y-1">
          <label class="text-sm font-medium">系统提示词</label>
          <Textarea v-model="editSystemPrompt" placeholder="描述角色设定、说话风格与边界。" />
        </div>
        <p v-if="updateError" class="text-sm text-destructive">{{ updateError }}</p>
      </div>
      <DialogFooter>
        <Button variant="outline" :disabled="updating" @click="editTarget = null">取消</Button>
        <Button :disabled="updating" @click="handleUpdateThemeCard">
          {{ updating ? "保存中..." : "保存" }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>

  <Dialog
    :open="deleteTarget !== null"
    @update:open="
      (v) => {
        if (!v) deleteTarget = null;
      }
    "
  >
    <DialogContent>
      <DialogHeader>
        <DialogTitle>删除 Theme Card</DialogTitle>
        <DialogDescription>
          确定要删除「{{
            deleteTarget?.name
          }}」吗？此操作将同时删除该卡片下的所有会话和消息，且不可撤销。
        </DialogDescription>
      </DialogHeader>
      <p v-if="deleteError" class="text-sm text-destructive">{{ deleteError }}</p>
      <DialogFooter>
        <Button variant="outline" :disabled="deleting" @click="deleteTarget = null">取消</Button>
        <Button variant="destructive" :disabled="deleting" @click="handleDeleteThemeCard">
          {{ deleting ? "删除中..." : "确认删除" }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
