<script setup lang="ts">
// Give some Markdown content and show a nice rendered output, can include code snippets
// to highlight with TreeSitter

import { onMounted, ref, Ref, watch } from 'vue';
import { commands } from '../ts/commands';

let props = defineProps<{
    // The Markdown content to render
    content: string,
    // An identifier of the content, so when it changes we can know if that's an update of the last content or another unrelated content
    contentId: string
}>()

const renderedHtml: Ref<null | string> = ref(null)

// This is null except during the generation of a content, we store the content id of the text being rendered
const contentIdInRendering: Ref<null | string> = ref(null)
const contentIdAtLastContentChange: Ref<null | string> = ref(props.contentId)

async function refreshContent(c: string) {
    console.log("sending for render", c)
    contentIdInRendering.value = props.contentId
    let result = await commands.renderMarkdownWithHighlighting(c)
    if (result.status == "ok") {
        // We have to make sure the result do still correspond to the content id we received before starting renderMarkdownWithHighlighting
        // because this method can take some time especially if we have a lot of different code snippets

        if (props.contentId == contentIdInRendering.value)
            renderedHtml.value = result.data
        // else we just ignore the result
    }
    contentIdInRendering.value = null
}

// On each props.code change, if the code is different that before, refresh it
watch(() => props.content, async (newContent, _oldContent) => {
    if (contentIdAtLastContentChange.value != props.contentId)
        renderedHtml.value = null // if the content id has changed, we don't want to see the previous content
    contentIdAtLastContentChange.value = props.contentId
    refreshContent(newContent)
})

onMounted(() => {
    contentIdAtLastContentChange.value = props.contentId
    refreshContent(props.content)
})
</script>

<template>
    <div v-if="!renderedHtml" class="text-blue-400">Loading Markdown content, if this is taking time, there is
        probably lots of code snippets to highlight...</div>
    <div class="text-xl markdown" v-html="renderedHtml"></div>
</template>
