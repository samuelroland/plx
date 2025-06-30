<script setup lang="ts">
import Home from "./Home.vue";
import Join from "./Join.vue";
import Start from "./Start.vue";
import { Ref, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Page } from "./types";


const page: Ref<Page> = ref("home")

async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    // greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<template>
    <div class="w-full h-[100vh]">
        <header class="h-7 bg-orange-200 flex px-2 p-1">
            <span class="font-bold">PLX</span>
        </header>
        <div class="h-[95vh] flex items-center justify-center">
            <Home v-if="page == 'home'" :join-live-session-fn="() => page = 'join'"
                :start-live-session-fn="() => page = 'start'"></Home>
            <Join v-if="page == 'join'"></Join>
            <Start v-if="page == 'start'"></Start>
        </div>
    </div>
</template>
