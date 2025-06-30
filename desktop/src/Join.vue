<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

async function getSessions() {
    const result = invoke("get_sessions")
}
</script>

<template>
    <div class="p-2 sm:mx-5 md:mx-10 lg:mx-10 max-w-[1300px] flex items-center flex-col">
        <h1>Join a session</h1>
        <span>Querying session list for current course</span>
    </div>
</template>
