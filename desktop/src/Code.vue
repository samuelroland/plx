<script setup lang="ts">
import { onMounted, ref, Ref, watch } from 'vue';
import { commands } from './ts/commands';

const highlightedCode: Ref<null | string> = ref(null)

let props = defineProps<{ code: string, path: string }>()

async function refreshCode(code: string) {
    // trim code to avoid the empty lines at the end to take place
    let result = await commands.highlightCodeWithTreeSitter(props.path, code.trim());
    if (result.status == "ok") {
        highlightedCode.value = result.data
    }
}

// On each props.code change, if the code is different that before, refresh it
watch(() => props.code, async (oldCode, newCode) => {
    if (oldCode.trim() != newCode.trim()) {
        refreshCode(newCode)
    }
})

onMounted(() => {
    refreshCode(props.code)
})
</script>

<template>
    <div class="relative">
        <div class="absolute top-0 right-0 px-3 py-1 text-gray-600">{{ props.path }}</div>
        <pre><code v-html="highlightedCode"></code></pre>
    </div>
</template>
