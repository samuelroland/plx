<script setup lang="ts">
import { Exo } from '../ts/commands';
import { ExoCheckResult, ExoStatusReport } from '../ts/shared';
import Markdown from './Markdown.vue';
import { anonymizeText } from "../util.ts"

defineProps<{ exo: Exo | undefined, exo_status?: ExoStatusReport | undefined }>()

// Display CLI arguments as string, use JSON.stringify in case it needs to have escapes and quotes shown
function argsify(args: string[]): string {
    console.log(args, JSON.stringify(args))
    return args.map(e => e.includes(" ") || e.includes("\n") || e.includes("\t") ? JSON.stringify(e) : e).join(" ")
}

function bgFromCheckResult(compilation_success: boolean, result: ExoCheckResult | undefined) {
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
    <span v-if="exo == undefined" class="text-red-500">Exo is undefined and cannot be rendered</span>
    <h1>{{ exo?.name }}</h1>
    <Markdown :content="exo?.instruction ?? ''" />

    <div v-if="exo_status?.compilation_running || exo_status?.compilation_success == false">
        <h2 v-if="exo_status?.compilation_running">Build</h2>
        <h2 v-if="!exo_status?.compilation_running && !exo_status?.compilation_success" class="!text-red-500">Build
            failed</h2>
        <pre class="p-2" v-html="anonymizeText(exo_status?.compilation_output ?? '')" />
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
                v-if="!exo_status?.check_results[idx] || exo_status?.check_results[idx] && exo_status?.check_results[idx].state.status.type != 'Passed'">
                <h4 v-if="check.args && check.args.length > 0">Arguments: <span
                        class="text-lg text-gray-500 font-mono">{{
                            argsify(check.args ?? []) }}</span></h4>
                <h4>Expected</h4>
                <pre class="px-2 py-1 rounded-md">{{ check.test.expected }}</pre>
            </div>

            <!-- If the exo status does exist, show the given output and the diff, except if the check has passed -->
            <div v-if="exo_status?.check_results[idx] && exo_status?.check_results[idx].state.status.type != 'Passed'">
                <div v-if="exo_status">
                    <div v-if="exo_status?.check_results[idx] && exo_status?.check_results[idx].output.length == 0">
                        <h4 class="inline">Given</h4>
                        <!-- <pre v-if="exo_status?.check_results[idx].output.length > 0" class="px-2 py-1 rounded-md">{{ -->
                        <!--     exo_status?.check_results[idx].output.join("\n") }}</pre> -->
                        <span class="ml-3 text-gray-800">
                            <em>empty</em>
                        </span>
                    </div>

                    <div
                        v-if="exo_status?.check_results[idx] && exo_status?.check_results[idx].state.status.type == 'Failed' && exo_status?.check_results[idx].output.length > 0">
                        <h4>Diff</h4>

                        <!-- <span class="diff-minus">Given</span> <span class="diff-plus">Expected</span> -->
                        <pre class="rounded-md" v-html="exo_status?.check_results[idx].state.status.content.diff"></pre>
                    </div>
                </div>
            </div>
        </div>
    </div>

</template>
