<script setup lang="ts">
import { ref, Ref } from "vue";
import { commands, ProjectInfo, Session } from "./bindings";

let course: Ref<Exo | null> = ref(null)

type Answer = string
const answers: Ref<Map<number, Answer>> | null = ref(new Map())

async function getExo() {

}

</script>

<template>
    <div class="p-2 sm:mx-5 md:mx-10 lg:mx-10 max-w-[1300px] flex items-center flex-col">
        <h1>Join a session</h1>
        <span>Querying session list for current course</span>
    </div>
</template>
