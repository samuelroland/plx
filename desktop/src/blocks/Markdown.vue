<script setup lang="ts">
// Give some Markdown content and show a nice rendered output, can include code snippets
// to highlight with TreeSitter

import { onMounted, ref, Ref, watch } from 'vue';
import { commands } from '../ts/commands';

const renderedHtml: Ref<null | string> = ref(null)

let props = defineProps<{ content: string }>()

async function refreshContent(c: string) {
    console.log("sending for render", c)
    let result = await commands.renderMarkdownWithHighlighting(c)
    if (result.status == "ok") {
        renderedHtml.value = result.data
    }
}

// On each props.code change, if the code is different that before, refresh it
watch(() => props.content, async (newContent, _oldContent) => {
    refreshContent(newContent)
})

onMounted(() => {
    refreshContent(props.content)
})
</script>

<template>
    <div class="text-xl" v-html="renderedHtml"></div>
</template>
