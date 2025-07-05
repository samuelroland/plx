<script setup lang="ts">
import Home from "./pages/Home.vue";
import Dashboard from "./pages/Dashboard.vue";
import Course from "./pages/Course.vue";
import { useLiveStore } from "./stores/LiveStore";
import { useGlobalStore } from "./stores/GlobalStore";
import { onMounted, ref } from "vue";
import { commands } from "./ts/commands";

const live = useLiveStore()
const global = useGlobalStore()
const style = ref("")

onMounted(async () => {
    const raw_css = await commands.loadDefaultThemeCss()
    style.value = "<style>" + raw_css + "</style>"
})
</script>

<template>
    <div v-html="style"></div>
    <div class="w-full h-[100vh]">
        <header class="h-7 bg-orange-200 flex px-3 p-1 items-center">
            <span class="font-bold flex-1">PLX</span>
            <span v-if="live.session">Current session: {{ live.session.name }} - {{ live.role }} - number {{
                live.client_num }} </span>
            <span v-else class="text-gray-600/50 italic">No live session</span>
        </header>
        <div class="h-[95vh] flex items-center justify-center">
            <Home v-if="global.page == 'home'">
            </Home>
            <Join v-if="global.page == 'join'"></Join>
            <Start v-if="global.page == 'start'"></Start>
            <Dashboard v-if="global.page == 'dashboard'"></Dashboard>
            <Course v-if="global.page == 'course'"></Course>
        </div>
    </div>
</template>
