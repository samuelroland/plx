<script setup lang="ts">
import { Exo } from './ts/commands';
import Markdown from './Markdown.vue';

const props = defineProps<{ exo: Exo | undefined }>()

// Display CLI arguments as string, use JSON.stringify in case it needs to have escapes and quotes shown
function argsify(args: string[]): string {
    console.log(args, JSON.stringify(args))
    return args.map(e => e.includes(" ") || e.includes("\n") || e.includes("\t") ? JSON.stringify(e) : e).join(" ")
}

</script>

<template>
    <span v-if="exo == undefined" class="text-red-500">Exo is undefined and cannot be rendered</span>
    <h1>{{ exo?.name }}</h1>
    <Markdown :content="exo?.instruction ?? ''"></Markdown>

    <div>
        <h2>Checks</h2>
        <div v-for="(check, idx) in exo?.checks">
            <h3><span class="font-bold">C{{ idx + 1 }}:</span> {{ check.name }}</h3>
            <h4 v-if="check.args && check.args.length > 0">Arguments: <span class="text-lg text-gray-500 font-mono">{{
                argsify(check.args
                    ?? []) }}</span></h4>
            <h4>Expected</h4>
            <pre class="p-3 rounded-md">{{ check.test.expected }}</pre>
        </div>
    </div>
</template>
