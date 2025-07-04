<script setup lang="ts">
import { Answer } from './stores/LiveStore';
import Code from './Code.vue';
import { ForwardedFile } from './ts/bindings';

let props = defineProps<{ answer: Answer }>()

function lastFileTime(files: Map<string, ForwardedFile>) {
    const maxTime = Math.max(...Array.from(files.values()).map(file => file.time))
    return formatTimestampAsHoursMinutesSeconds(maxTime)
}

function formatTimestampAsHoursMinutesSeconds(time: number): string {
    const date = new Date(time * 1000)
    function normalize(number: number) {
        let result = number.toString()
        if (number < 10) {
            result = "0" + number
        }
        return result
    }
    return normalize(date.getHours()) + ":" + normalize(date.getUTCMinutes()) + ":" + normalize(date.getSeconds())
}

</script>

<template>
    <div>
        <div class="flex">
            <div>{{ answer.client_num }} at {{ lastFileTime(answer.files) }}</div>
        </div>
        <div v-for="file in answer.files.values()">
            <div class="mb-2">
                <Code :key="file.path" :code="file.content" :path="file.path"></Code>
            </div>
        </div>
    </div>
</template>
