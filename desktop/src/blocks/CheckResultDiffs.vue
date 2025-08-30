<script setup lang="ts">
import { ref } from 'vue';
import { ExoCheckResultWithOutput } from '../ts/shared';
import Code from './Code.vue';

defineProps<{ result: ExoCheckResultWithOutput, expected: string }>()

enum DiffMode {
    GivenExpected,
    MergedDiff
    // TODO: side by side diff ?
}

type DiffModeWithName = { mode: DiffMode, name: string }

const diffMode = ref(DiffMode.MergedDiff)
const diffModes: DiffModeWithName[] = [{ mode: DiffMode.GivenExpected, name: "Side by side" },
{ mode: DiffMode.MergedDiff, name: "Merged diff" }]

</script>

<template>
    <div class="flex space-x-2">
        <div v-for="mode of diffModes" :class="diffMode == mode.mode ? 'bg-blue-300' : 'bg-blue-100'"
            class="rounded-sm px-2 cursor-pointer" @click="diffMode = mode.mode">
            {{ mode.name }}
        </div>
    </div>

    <div v-if="diffMode == DiffMode.GivenExpected">
        <div class="flex flex-wrap space-x-5 text-sm lg:text-base">
            <div>
                <h4>Given</h4>
                <Code v-if="result.output.length > 0" :code="result.output.join('\n')"></Code>
                <span v-else class="ml-3 text-gray-800"> <em>empty</em> </span>
            </div>
            <div>
                <h4>Expected</h4>
                <Code :code="expected.trim()"></Code>
            </div>
        </div>
    </div>

    <div v-if="diffMode == DiffMode.MergedDiff">
        <div v-if="result.state.status.type == 'Failed' && result.output.length > 0">
            <div class="flex items-center">
                <h4>Diff</h4>
                <span class="ml-10">Given (-)</span>
                <span class="ml-5">Expected (+)</span>
            </div>

            <Code :rawAsHtml="true" :code="result.state.status.content.diff"></Code>
        </div>
        <div v-else>
            <h4>Expected (No diff available)</h4>
            <Code :code="expected.trim()"></Code>
        </div>
    </div>

</template>
