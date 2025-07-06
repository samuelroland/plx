<script setup lang="ts">
import { Exo, ExoCheckResult, ExoStatusReport } from '../ts/commands';
import Code from './Code.vue';
import Markdown from './Markdown.vue';
import Convert from 'ansi-to-html';

const props = defineProps<{ exo: Exo | undefined, exo_status: ExoStatusReport | undefined }>()

// Display CLI arguments as string, use JSON.stringify in case it needs to have escapes and quotes shown
function argsify(args: string[]): string {
    console.log(args, JSON.stringify(args))
    return args.map(e => e.includes(" ") || e.includes("\n") || e.includes("\t") ? JSON.stringify(e) : e).join(" ")
}

function bgFromCheckResult(result: ExoCheckResult | undefined) {
    if (!result) return

    switch (result.state.status.type) {
        case "Passed": return "bg-green-200"
        case "RunFail": return "bg-purple-100"
        case 'Failed': return "bg-orange-200"
    }
    return ""
}

function ansiToHtml(ansi: string) {
    var convert = new Convert();
    return convert.toHtml(ansi);
}

</script>

<template>
    <span v-if="exo == undefined" class="text-red-500">Exo is undefined and cannot be rendered</span>
    <h1>{{ exo?.name }}</h1>
    <Markdown :content="exo?.instruction ?? ''"></Markdown>

    <div>
        <h2>Checks</h2>
        <div v-for="(check, idx) in exo?.checks">
            <h3 :class="bgFromCheckResult(exo_status?.check_results[idx])">
                <span class="font-bold">C{{ idx + 1 }}:</span>
                {{ check.name }}
            </h3>
            <h4 v-if="check.args && check.args.length > 0">Arguments: <span class="text-lg text-gray-500 font-mono">{{
                argsify(check.args
                    ?? []) }}</span></h4>
            <div v-if="exo_status?.check_results[idx].state.status.type != 'Passed'">
                <div v-if="exo_status?.check_results[idx].state.status.type != 'Failed'">
                    <h4>Expected</h4>
                    <pre class="p-3 rounded-md">{{ check.test.expected }}</pre>

                    <h4>Given</h4>
                    <pre class="p-3 rounded-md">{{ exo_status?.check_results[idx].output.join("\n") }}</pre>
                </div>

                <h4>Diff - <span class="diff-minus">Given</span> <span class="diff-plus">Expected</span></h4>
                <div class="p-3 rounded-md" v-if="exo_status?.check_results[idx].state.status.type == 'Failed'"
                    v-html="exo_status?.check_results[idx].state.status.content.diff">
                </div>
            </div>
        </div>
    </div>


    <div v-if="exo_status?.compilation_running || exo_status?.compilation_success == false">
        <h3 v-if="exo_status?.compilation_running">Build</h3>
        <h3 v-if="!exo_status?.compilation_running && !exo_status?.compilation_success" class="text-red-500">Build
            failed</h3>
        <pre v-html="ansiToHtml(exo_status?.compilation_output.join('\n') ?? '')" />
    </div>

    {{ exo_status }}
</template>
