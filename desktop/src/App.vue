<script setup lang="ts">
import Home from "./Home.vue";
import Join from "./Join.vue";
import Start from "./Start.vue";
import Dashboard from "./Dashboard.vue";
import { useLiveStore } from "./stores/LiveStore";
import { useGlobalStore } from "./stores/GlobalStore";

const live = useLiveStore()
const global = useGlobalStore()
</script>

<template>
    <div class="w-full h-[100vh]">
        <header class="h-7 bg-orange-200 flex px-2 p-1 items-center">
            <span class="font-bold flex-1">PLX</span>
            <span v-if="live.session">{{ live.session.name }}</span>
            <span v-else class="text-gray-600/50 italic">No live session</span>
        </header>
        <div class="h-[95vh] flex items-center justify-center">
            <Home v-if="global.page == 'home'">
            </Home>
            <Join v-if="global.page == 'join'"></Join>
            <Start v-if="global.page == 'start'"></Start>
            <Dashboard v-if="global.page == 'dashboard'"></Dashboard>
        </div>
    </div>
</template>
