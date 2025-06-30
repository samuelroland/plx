<script setup lang="ts">
import Home from "./Home.vue";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const greetMsg = ref("");
const name = ref("");

async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<template>
    <header class="h-7 bg-orange-200 flex px-2 p-1">
        <span class="font-bold">PLX</span>
    </header>
    <div class="w-full h-[100vh] flex items-center justify-center">
        <Home></Home>
    </div>
</template>
