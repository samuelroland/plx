<script setup lang="ts">
import { useTrainStore } from '../stores/TrainStore';
import { onKeyStroke } from "@vueuse/core"
import CodeExo from '../blocks/CodeExo.vue';
import { useGlobalStore } from '../stores/GlobalStore';
import { onMounted } from 'vue';

const train = useTrainStore()
const global = useGlobalStore()

onKeyStroke(['Escape'], (_) => {
    global.page = "course"
})

onMounted(() => {
    train.startExo()
})

</script>

<template>
    <div class="h-full w-full px-5">
        <CodeExo :exo="train.currentExo()"></CodeExo>
    </div>
</template>
