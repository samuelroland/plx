<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useTrainStore } from './stores/TrainStore';
import { Exo, Skill } from './ts/commands';
import { onKeyStroke } from "@vueuse/core"
import Markdown from './Markdown.vue';

const props = defineProps<{ exo: Exo | undefined }>()

</script>

<template>
    <span v-if="exo == undefined" class="text-red-500">Exo is undefined and cannot be rendered</span>
    <h1>{{ exo?.name }}</h1>
    <Markdown :content="exo?.instruction ?? ''"></Markdown>

    <div>
        <h2>Checks</h2>
        <div v-for="check in exo?.checks">
            <h3>{{ check.name }}</h3>
            <h4 v-if="check.args && check.args.length > 0">Arguments: {{ check.args.join(" ") }}</h4>
            <h4>Expected: <span class="text-orange-400">{{ check.test.expected }}</span></h4>
        </div>
    </div>
</template>
