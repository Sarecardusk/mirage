<script setup lang="ts">
import { onMounted, ref } from "vue";
import AppLayout from "@/components/AppLayout.vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  getLlmApiKey,
  getLlmConfig,
  listLlmModels,
  setLlmConfig,
  testLlmConnection,
} from "@/services/config";

const endpoint = ref("");
const apiKey = ref("");
const model = ref("");
const temperatureEnabled = ref(false);
const temperatureVal = ref(1);
const maxTokensEnabled = ref(false);
const maxTokensVal = ref(2048);
const topPEnabled = ref(false);
const topPVal = ref(1);
const frequencyPenaltyEnabled = ref(false);
const frequencyPenaltyVal = ref(0);
const presencePenaltyEnabled = ref(false);
const presencePenaltyVal = ref(0);
const loading = ref(false);
const saving = ref(false);
const fetchingModels = ref(false);
const testingConnection = ref(false);
const modelPanelOpen = ref(false);
const modelOptions = ref<string[]>([]);
const error = ref("");
const savedMessage = ref("");
const infoMessage = ref("");

function clearRuntimeMessages() {
  error.value = "";
  savedMessage.value = "";
  infoMessage.value = "";
}

function paramValue(enabled: boolean, val: number): number | null {
  return enabled ? val : null;
}

async function loadConfig() {
  loading.value = true;
  error.value = "";
  try {
    const [config, plaintextApiKey] = await Promise.all([getLlmConfig(), getLlmApiKey()]);
    endpoint.value = config.endpoint;
    apiKey.value = plaintextApiKey;
    model.value = config.model;

    temperatureEnabled.value = config.temperature !== null;
    temperatureVal.value = config.temperature ?? 1;
    maxTokensEnabled.value = config.maxTokens !== null;
    maxTokensVal.value = config.maxTokens ?? 2048;
    topPEnabled.value = config.topP !== null;
    topPVal.value = config.topP ?? 1;
    frequencyPenaltyEnabled.value = config.frequencyPenalty !== null;
    frequencyPenaltyVal.value = config.frequencyPenalty ?? 0;
    presencePenaltyEnabled.value = config.presencePenalty !== null;
    presencePenaltyVal.value = config.presencePenalty ?? 0;
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
      temperature: paramValue(temperatureEnabled.value, temperatureVal.value),
      maxTokens: paramValue(maxTokensEnabled.value, maxTokensVal.value),
      topP: paramValue(topPEnabled.value, topPVal.value),
      frequencyPenalty: paramValue(frequencyPenaltyEnabled.value, frequencyPenaltyVal.value),
      presencePenalty: paramValue(presencePenaltyEnabled.value, presencePenaltyVal.value),
    });
    savedMessage.value = "保存成功";
  } catch (saveError) {
    error.value = saveError instanceof Error ? saveError.message : String(saveError);
  } finally {
    saving.value = false;
  }
}

async function fetchModelList() {
  clearRuntimeMessages();
  fetchingModels.value = true;
  modelPanelOpen.value = false;
  try {
    const models = await listLlmModels({
      endpoint: endpoint.value,
      apiKey: apiKey.value,
    });
    if (models.length === 0) {
      throw new Error("模型列表为空，请确认 endpoint 与 apiKey 是否正确。");
    }
    modelOptions.value = models;
    modelPanelOpen.value = true;
    infoMessage.value = `已获取 ${models.length} 个模型`;
  } catch (fetchError) {
    error.value = fetchError instanceof Error ? fetchError.message : String(fetchError);
  } finally {
    fetchingModels.value = false;
  }
}

function chooseModel(modelId: string) {
  model.value = modelId;
  modelPanelOpen.value = false;
}

async function testRequest() {
  clearRuntimeMessages();
  testingConnection.value = true;
  try {
    await testLlmConnection({
      endpoint: endpoint.value,
      apiKey: apiKey.value,
      model: model.value,
    });
    infoMessage.value = "测试请求成功";
  } catch (testError) {
    error.value = testError instanceof Error ? testError.message : String(testError);
  } finally {
    testingConnection.value = false;
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
      <p class="text-sm text-muted-foreground">配置落盘为 apiKeyRef，密钥明文仅存于本地 Vault。</p>

      <div class="space-y-1">
        <label class="text-sm font-medium">API Endpoint</label>
        <Input
          v-model="endpoint"
          placeholder="https://api.deepseek.com"
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
        <div class="relative space-y-2">
          <div class="flex items-center gap-2">
            <Input v-model="model" placeholder="deepseek-chat" :disabled="loading || saving" />
            <Button
              variant="outline"
              :disabled="loading || saving || fetchingModels || !endpoint.trim() || !apiKey.trim()"
              @click="fetchModelList"
            >
              {{ fetchingModels ? "获取中..." : "获取模型列表" }}
            </Button>
            <Button
              variant="outline"
              :disabled="
                loading ||
                saving ||
                testingConnection ||
                !endpoint.trim() ||
                !apiKey.trim() ||
                !model.trim()
              "
              @click="testRequest"
            >
              {{ testingConnection ? "测试中..." : "测试请求" }}
            </Button>
          </div>

          <div
            v-if="modelPanelOpen"
            class="absolute z-20 mt-1 max-h-56 w-full overflow-y-auto rounded-md border bg-background p-1 shadow"
          >
            <button
              v-for="option in modelOptions"
              :key="option"
              class="block w-full rounded px-3 py-2 text-left text-sm hover:bg-accent"
              type="button"
              @click="chooseModel(option)"
            >
              {{ option }}
            </button>
          </div>
        </div>
        <p class="text-xs text-muted-foreground">
          DeepSeek 优先兼容：推荐使用 `deepseek-chat`，推理模式可选 `deepseek-reasoner`。
        </p>
      </div>

      <div class="space-y-2 pt-2">
        <h4 class="text-sm font-semibold">生成参数</h4>
        <p class="text-xs text-muted-foreground">勾选后启用；未勾选则使用供应商默认值。</p>

        <div class="space-y-2">
          <div class="flex items-center gap-3">
            <input
              id="temperatureEnable"
              v-model="temperatureEnabled"
              type="checkbox"
              :disabled="loading || saving"
              class="cursor-pointer"
            />
            <label
              for="temperatureEnable"
              class="w-52 cursor-pointer select-none text-sm font-medium"
            >
              Temperature (0–2)
            </label>
            <Input
              v-model.number="temperatureVal"
              class="w-28"
              type="number"
              step="0.1"
              min="0"
              max="2"
              :disabled="!temperatureEnabled || loading || saving"
            />
          </div>

          <div class="flex items-center gap-3">
            <input
              id="maxTokensEnable"
              v-model="maxTokensEnabled"
              type="checkbox"
              :disabled="loading || saving"
              class="cursor-pointer"
            />
            <label
              for="maxTokensEnable"
              class="w-52 cursor-pointer select-none text-sm font-medium"
            >
              Max Tokens
            </label>
            <Input
              v-model.number="maxTokensVal"
              class="w-28"
              type="number"
              min="1"
              :disabled="!maxTokensEnabled || loading || saving"
            />
          </div>

          <div class="flex items-center gap-3">
            <input
              id="topPEnable"
              v-model="topPEnabled"
              type="checkbox"
              :disabled="loading || saving"
              class="cursor-pointer"
            />
            <label for="topPEnable" class="w-52 cursor-pointer select-none text-sm font-medium">
              Top P (0–1)
            </label>
            <Input
              v-model.number="topPVal"
              class="w-28"
              type="number"
              step="0.01"
              min="0"
              max="1"
              :disabled="!topPEnabled || loading || saving"
            />
          </div>

          <div class="flex items-center gap-3">
            <input
              id="frequencyPenaltyEnable"
              v-model="frequencyPenaltyEnabled"
              type="checkbox"
              :disabled="loading || saving"
              class="cursor-pointer"
            />
            <label
              for="frequencyPenaltyEnable"
              class="w-52 cursor-pointer select-none text-sm font-medium"
            >
              Frequency Penalty (−2–2)
            </label>
            <Input
              v-model.number="frequencyPenaltyVal"
              class="w-28"
              type="number"
              step="0.1"
              min="-2"
              max="2"
              :disabled="!frequencyPenaltyEnabled || loading || saving"
            />
          </div>

          <div class="flex items-center gap-3">
            <input
              id="presencePenaltyEnable"
              v-model="presencePenaltyEnabled"
              type="checkbox"
              :disabled="loading || saving"
              class="cursor-pointer"
            />
            <label
              for="presencePenaltyEnable"
              class="w-52 cursor-pointer select-none text-sm font-medium"
            >
              Presence Penalty (−2–2)
            </label>
            <Input
              v-model.number="presencePenaltyVal"
              class="w-28"
              type="number"
              step="0.1"
              min="-2"
              max="2"
              :disabled="!presencePenaltyEnabled || loading || saving"
            />
          </div>
        </div>
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
      <p v-if="infoMessage" class="text-sm text-green-600">{{ infoMessage }}</p>
      <p v-if="error" class="text-sm text-destructive">{{ error }}</p>
    </div>
  </AppLayout>
</template>
