<script setup lang="ts">
import {ref, onMounted, computed} from "vue";
import {invoke} from "@tauri-apps/api/core";

interface AutofillItem {
  name: string;
  value: string;
}

const data = ref<Array<any>>([]);
const selectedItems = ref<AutofillItem[]>([]);
const keyword = ref<string>("");

async function loadData() {
  data.value = [];
  setTimeout(async () => {
    data.value = await invoke("get_table_data", {tableName: "autofill"})
  }, 500);
}

const dataView = computed(() => {
  let res = data.value
  if (keyword.value) {
    res = res.filter(item => item.value.toLowerCase().includes(keyword.value.toLowerCase()));
  }
  return res;
})

async function reloadData() {
  keyword.value = "";
  selectedItems.value = [];
  await loadData();
}

function toggleSelect(item: any) {
  if (isInclude(selectedItems.value, item)) {
    selectedItems.value = removeItem(selectedItems.value, item)
  } else {
    selectedItems.value.push(item);
  }
}

const isSelectedAll = computed(() => {
  return keyword.value && dataView.value.length > 0
      ? selectedItems.value.length === dataView.value.length
      : selectedItems.value.length === data.value.length
})

function toggleSelectAll() {
  if (!isSelectedAll.value) {
    selectedItems.value = keyword.value ? dataView.value : data.value;
  } else {
    selectedItems.value = []
  }
}

async function deleteMany() {
  if (selectedItems.value.length === 0) return;
  const itemsToDelete = Array.from(selectedItems.value);
  const promises = itemsToDelete.map(item => invoke("delete_record", {
    tableName: "autofill",
    name: item.name,
    value: item.value,
  }));
  await Promise.all(promises);
  selectedItems.value = []
  await loadData();
}

async function deleteOne(item: any) {
  await invoke("delete_record", {
    tableName: "autofill",
    name: item.name,
    value: item.value,
  });
  selectedItems.value = removeItem(selectedItems.value, item);
  data.value = removeItem(data.value, item);
}

function clearKeyword() {
  keyword.value = "";
}

function isInclude(list: AutofillItem[], item: AutofillItem): boolean {
  return list.some(it => it.value === item.value && it.name === item.name);
}

function removeItem(list: AutofillItem[], item: AutofillItem): AutofillItem[] {
  return list.filter(it => it.value !== item.value || it.name !== item.name);
}

onMounted(async () => {
  await loadData();
});
</script>

<template>
  <main class="container">
    <div class="header">
      <div class="actions">
        <button
            class="load-btn"
            @click="reloadData"
        >
          Reload
        </button>
        <button
            class="select-all-btn"
            @click="toggleSelectAll"
            :class="{ active: isSelectedAll }"
            :title="keyword ? 'Select All Search Result' : ''"
        >
          {{ isSelectedAll ? 'Deselect All' : 'Select All' }}
        </button>
        <button
            class="delete-btn"
            @click="deleteMany"
            :disabled="selectedItems.length === 0"
        >
          Delete ({{ selectedItems.length }})
        </button>
        <div class="input-wrapper">
          <input class="search-input" v-model="keyword" type="text" placeholder="Search..."/>
          <span class="clear-mark" :class="{hidden: keyword.length===0}" @click="clearKeyword">×</span>
        </div>
      </div>
    </div>
    <div class="list">
      <div
          class="item"
          v-for="d in dataView"
          :key="`${d.name}-${d.value}`"
          @click="toggleSelect(d)"
      >
        <div class="item-content">
          <input
              type="checkbox"
              :checked="isInclude(selectedItems, d) "
              @click.stop="toggleSelect(d)"
          />
          <span class="value">{{ d.value }}</span>
          <button class="delete-item-btn" @click.stop="deleteOne(d)">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 6h18"></path>
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
            </svg>
          </button>
        </div>
      </div>
      <div v-if="data.length === 0" class="empty-state">
        <div>Empty Here.</div>
        <div class="danger">Please Close Edge Browser And Reload Again.</div>
      </div>
    </div>
  </main>
</template>

<style scoped>
.container {
  width: 500px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
}

.header {
  margin-top: 20px;
  margin-bottom: 20px;
  padding-bottom: 15px;
  border-bottom: 1px solid #333;
}

.header h1 {
  margin: 0 0 15px 0;
  font-size: 24px;
  color: #f0f0f0;
}

.actions {
  display: flex;
  justify-content: space-around;
  gap: 10px;
}

button {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s ease;
}

.load-btn {
  background-color: #333;
  color: #e0e0e0;
}

.select-all-btn {
  background-color: #333;
  color: #e0e0e0;
}

.select-all-btn:hover {
  background-color: #555;
}

.select-all-btn.active {
  background-color: #666;
}

.delete-btn {
  background-color: #d32f2f;
  color: white;
}

.delete-btn:hover:not(:disabled) {
  background-color: #f44336;
}

.delete-btn:disabled {
  background-color: #5d4037;
  cursor: not-allowed;
  opacity: 0.7;
}

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.search-input {
  width: 150px;
  padding: 10px; /* 左侧留出图标空间 */
  font-size: 16px;
  border: none;
  border-radius: 30px; /* 圆角 */
  background-color: #2d2d2d; /* 深色背景 */
  color: #e0e0e0; /* 浅色文字 */
  transition: all 0.3s ease;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.2);
  outline: none;
}

.clear-mark {
  position: absolute;
  right: 15px;
  top: -5px;
  cursor: pointer;
  font-size: 30px;
}

.clear-mark:hover {
  color: #aaa;
}

.hidden {
  display: none;
}

.list {
  height: calc(100vh - 120px);
  background-color: #222;
  border-radius: 6px;
  overflow: auto;
}

.item {
  padding: 12px 15px;
  border-bottom: 1px solid #333;
  transition: background-color 0.2s ease;
}

.item:hover {
  background-color: #2a2a2a;
}

.item-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.item input[type="checkbox"] {
  accent-color: #4caf50;
  width: 16px;
  height: 16px;
  cursor: pointer;
}

.value {
  flex: 1;
  text-align: left;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.delete-item-btn {
  background: none;
  border: none;
  color: #ff6b6b;
  padding: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.item:hover .delete-item-btn {
  opacity: 1;
}

.delete-item-btn:hover {
  color: #ff5252;
}

.danger {
  color: #ff5252;
}

.empty-state {
  padding: 20px;
  text-align: center;
  color: #888;
  font-style: italic;
  font-size: 20px;
  font-weight: bolder;
}

/* 全局滚动条美化 */
::-webkit-scrollbar {
  width: 10px; /* 垂直滚动条宽度 */
  height: 10px; /* 水平滚动条高度 */
}

/* 滚动条轨道 */
::-webkit-scrollbar-track {
  background: #2a2a2a; /* 轨道颜色 */
  border-radius: 5px; /* 圆角 */
}

/* 滚动条滑块 */
::-webkit-scrollbar-thumb {
  background: #4a4a4a; /* 滑块颜色 */
  border-radius: 5px; /* 圆角 */
  border: 2px solid #2a2a2a; /* 轨道与滑块间的边框 */
}

/* 滚动条滑块悬停状态 */
::-webkit-scrollbar-thumb:hover {
  background: #5a5a5a; /* 悬停时颜色变亮 */
}

/* 滚动条滑块激活状态 */
::-webkit-scrollbar-thumb:active {
  background: #6a6a6a; /* 点击时颜色更亮 */
}
</style>