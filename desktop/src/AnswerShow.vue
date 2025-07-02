<script setup lang="ts">
import { Answer } from './ts/bindings';


let props = defineProps<{ answer: Answer }>()

</script>

<template>
    <div>
        <span>ClientNum {{ answer.client_num }}</span>
        <div v-for="file in answer.files.values()">
            <div>
                <span>{{ file.time }} - {{ file.path }}</span>
                <Code :code="file.content"></Code>
            </div>
        </div>
    </div>
</template>
