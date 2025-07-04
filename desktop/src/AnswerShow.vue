<script setup lang="ts">

import { Answer } from './stores/LiveStore';
import Code from './Code.vue';
import { CheckStatus, ExoCheckResult, ForwardedFile } from './ts/bindings';
import CheckResultBox from './CheckResultBox.vue';

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

function sortCheckResults(results: Map<number, ExoCheckResult>) {
    return Array.from(results.values()).sort((a, b) => a.index - b.index)
}
</script>

<template>

    <div>
        <!-- header of the answer -->
        <div class="flex">
            <div class="flex-1"><span class="font-bold">{{ answer.client_num }}</span> at {{ lastFileTime(answer.files)
                }}</div>
            <div v-for="(result, idx) in sortCheckResults(answer.checks_status)">
                <CheckResultBox :check="result"></CheckResultBox>
            </div>
        </div>

        <!-- code files -->
        <div z-index="2" v-for="file in answer.files.values()">
            <div class="mb-2">
                <Code :key="file.path" :code="file.content" :path="file.path"></Code>
            </div>
        </div>
    </div>
</template>
