<script setup lang="ts">
import { onMounted, ref } from "vue";
import AppLayout from "@/components/AppLayout.vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { getLlmConfig, setLlmConfig } from "@/services/config";

const endpoint = ref("");
const apiKey = ref("");
const model = ref("");
const loading = ref(false);
const saving = ref(false);
const error = ref("");
const savedMessage = ref("");

async function loadConfig() {
  loading.value = true;
  error.value = "";
  try {
    const config = await getLlmConfig();
    endpoint.value = config.endpoint;
    apiKey.value = config.apiKey;
    model.value = config.model;
  } catch (loadError) {
    error.value = loadError instanceof Error ? loadError.message : String(loadError);
  } finally {
    loading.value = false;
  }
}

async function saveConfig() {
  saving.value = true;
  error.value = "";
  savedMessage.value = "";
  try {
    await setLlmConfig({
      endpoint: endpoint.value,
      apiKey: apiKey.value,
      model: model.value,
    });
    savedMessage.value = "保存成功";
  } catch (saveError) {
    error.value = saveError instanceof Error ? saveError.message : String(saveError);
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  loadConfig();
});
</script>

<template>
  <AppLayout>
    <div class="max-w-2xl space-y-4 rounded-lg border p-6">
      <h3 class="text-lg font-semibold">LLM API 设置</h3>
      <p class="text-sm text-muted-foreground">MVP 阶段配置仅保存在内存中，重启后需要重新填写。</p>

      <div class="space-y-1">
        <label class="text-sm font-medium">API Endpoint</label>
        <Input
          v-model="endpoint"
          placeholder="https://api.openai.com/v1"
          :disabled="loading || saving"
        />
      </div>

      <div class="space-y-1">
        <label class="text-sm font-medium">API Key</label>
        <Input
          v-model="apiKey"
          type="password"
          placeholder="sk-..."
          :disabled="loading || saving"
        />
      </div>

      <div class="space-y-1">
        <label class="text-sm font-medium">Model</label>
        <Input v-model="model" placeholder="gpt-4o-mini" :disabled="loading || saving" />
      </div>

      <div class="flex items-center gap-3">
        <Button :disabled="loading || saving" @click="saveConfig">
          {{ saving ? "保存中..." : "保存配置" }}
        </Button>
        <Button variant="outline" :disabled="loading || saving" @click="loadConfig"
          >重新加载</Button
        >
      </div>

      <p v-if="savedMessage" class="text-sm text-green-600">{{ savedMessage }}</p>
      <p v-if="error" class="text-sm text-destructive">{{ error }}</p>
    </div>
  </AppLayout>
</template>
