<script setup lang="ts">
import { useSchemaStore } from '../../stores/schema'
import SchemaTreeNode from './SchemaTreeNode.vue'

const schemaStore = useSchemaStore()
</script>

<template>
  <div class="schema-panel">
    <div class="schema-panel__title">Schema</div>
    <div v-if="schemaStore.schemaTree.children.length === 0" class="schema-panel__empty">
      Schema yuklu degil
    </div>
    <div v-else class="schema-panel__tree">
      <SchemaTreeNode
        v-for="child in schemaStore.schemaTree.children"
        :key="child.path"
        :node="child"
        :depth="0"
      />
    </div>
  </div>
</template>

<style scoped>
.schema-panel {
  padding: 12px 0;
}

.schema-panel__title {
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 10px;
  padding: 0 12px;
}

.schema-panel__empty {
  padding: 20px 12px;
  color: #94a3b8;
  font-size: 13px;
  text-align: center;
}

.schema-panel__tree {
  display: flex;
  flex-direction: column;
}
</style>
