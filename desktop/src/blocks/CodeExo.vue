<script setup lang="ts">
import { Exo } from '../ts/commands';
import { ExoCheckResultWithOutput, ExoStatusReport } from '../ts/shared';
import Markdown from './Markdown.vue';
import { anonymizeAndSimplifyText } from "../util.ts"
import { useTrainStore } from '../stores/TrainStore.ts';
import Code from './Code.vue';
import { useGlobalStore } from '../stores/GlobalStore.ts';
import CheckResultDiffs from './CheckResultDiffs.vue';

const train = useTrainStore()
const global = useGlobalStore()

defineProps<{ exo: Exo | undefined, exo_status?: ExoStatusReport | undefined }>()

// Display CLI arguments as string, use JSON.stringify in case it needs to have escapes and quotes shown
function argsify(args: string[]): string {
    return args.map(e => e.includes(" ") || e.includes("\n") || e.includes("\t") ? JSON.stringify(e) : e).join(" ")
}

function bgFromCheckResult(compilation_success: boolean, result: ExoCheckResultWithOutput | undefined) {
    if (!result || !compilation_success) return "bg-gray-100"

    switch (result.state.status.type) {
        case "Passed": return "bg-green-200"
        case "RunFail": return "bg-purple-100"
        case 'Failed': return "bg-orange-100"
    }
    return ""
}


</script>

<template>
    <div v-if="exo == undefined" class="h-full w-full flex items-center justify-center">
        <span v-if="train.in_live_session" class="text-blue-500">
            Waiting for the first exo to start in this live session
        </span>
        <span v-else class="text-red-500">Exo is undefined and cannot be rendered</span>
    </div>
    <div v-else>
        <h1>{{ exo?.name }}</h1>
        <Markdown :content="exo?.instruction ?? ''" :contentId="exo.folder" />

        <div v-if="exo_status?.compilation_running || exo_status?.compilation_success == false">
            <h2 v-if="exo_status?.compilation_running">Build</h2>
            <h2 v-if="!exo_status?.compilation_running && !exo_status?.compilation_success" class="!text-red-500">Build
                failed</h2>
            <Code :rawAsHtml="true" :code="anonymizeAndSimplifyText(exo_status?.compilation_output ?? '')" />
        </div>

        <div>
            <h2>Checks</h2>
            <div v-for="(check, idx) in exo?.checks">
                <h3 class="px-1 rounded-sm"
                    :class="bgFromCheckResult(exo_status?.compilation_success ?? false, exo_status?.check_results[idx])">
                    <span class="font-bold">C{{ idx + 1 }}:</span>
                    {{ check.name }}
                </h3>

                <div
                    v-if="global.page != 'train' || exo_status?.check_results[idx] && exo_status?.check_results[idx].state.status.type != 'Passed'">
                    <h4 v-if="check.args && check.args.length > 0">
                        Arguments: <span class="text-lg text-gray-500 font-mono">{{ argsify(check.args ?? []) }}</span>
                    </h4>

                    <!-- Only show basic Expected output when the exo is not trained -->
                    <div v-if="global.page != 'train'">
                        <h4>Expected</h4>
                        <Code :code="check.test.expected"></Code>
                    </div>

                    <CheckResultDiffs v-if="exo_status?.check_results[idx]" :result="exo_status?.check_results[idx]"
                        :expected="check.test.expected">
                    </CheckResultDiffs>
                </div>
            </div>
        </div>
    </div>

</template>
