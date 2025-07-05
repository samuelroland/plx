<script setup lang="ts">
// Give some code to highlight, given a empty path, it is considered to be a raw output
// to not highglight
import { onMounted, ref, Ref, watch } from 'vue';
import { commands } from '../ts/commands';

const highlightedCode: Ref<null | string> = ref(null)
const rawOutput: Ref<null | string> = ref(null)

let props = defineProps<{ code: string, path?: string | null }>()

async function refreshCode(code: string) {
    if (props.path == null) {
        rawOutput.value = code
        highlightedCode.value = null
        return;
    }
    // trim code to avoid the empty lines at the end to take place
    let result = await commands.highlightCodeWithTreeSitter(props.path, code.trim());
    if (result.status == "ok") {
        highlightedCode.value = result.data
        rawOutput.value = null
    }
}

// On each props.code change, if the code is different that before, refresh it
watch(() => props.code, async (newCode, oldCode) => {
    if (oldCode.trim() != newCode.trim()) {
        refreshCode(newCode)
    }
})

onMounted(() => {
    refreshCode(props.code)
})
</script>

<template>
    <div v-if="rawOutput == null" class="relative">
        <div class="absolute top-0 right-0 px-3 py-1 text-gray-600">{{ props.path }}</div>
        <pre><code v-html="highlightedCode"></code></pre>
    </div>
    <div v-else>
        <pre><code>{{ rawOutput }}</code></pre>
    </div>
</template>
