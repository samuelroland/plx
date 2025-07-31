<script setup lang="ts">
import Home from "./pages/Home.vue";
import Dashboard from "./pages/Dashboard.vue";
import Course from "./pages/Course.vue";
import Train from "./pages/Train.vue";
import NotifZone from "./blocks/NotifZone.vue";
import { useLiveStore } from "./stores/LiveStore";
import { useGlobalStore } from "./stores/GlobalStore";
import { onMounted, ref } from "vue";
import { commands } from "./ts/commands";
import { useTrainStore } from "./stores/TrainStore";

const live = useLiveStore()
const global = useGlobalStore()
const train = useTrainStore()
const style = ref("")

onMounted(async () => {
    const raw_css = await commands.loadDefaultThemeCss()
    style.value = "<style>" + raw_css + "</style>"
})
</script>

<template>
    <div v-html="style"></div>
    <div class="w-full h-[100vh]">
        <header class="h-7 flex px-3 p-1 items-center" :class="live.isConnected ? 'bg-blue-200' : 'bg-orange-100'">
            <div class="flex-1 flex">
                <img class="w-8 max-w-[80vw] inline" src="/logo.svg" />
                <span :class="global.page == 'home' ? 'bg-blue-300' : ''"
                    class="px-1 mx-2 cursor-pointer hover:font-bold" @click="global.page = 'home'">Home</span>
                <span class="mx-2">-</span>
                <span :class="global.page == 'course' ? 'bg-blue-300' : ''" class="px-1 cursor-pointer hover:font-bold"
                    @click="global.page = 'course'">
                    {{ train.course?.name }}
                </span>
            </div>
            <span v-if="live.session">
                - Current session: {{ live.session.name }}
                - {{ live.role }}
                - Client number {{ live.client_num }} </span>
            <span v-else class="text-gray-600/80 italic">No live session</span>
        </header>
        <div class="h-[95vh] flex items-center justify-center">
            <Home v-if="global.page == 'home'">
            </Home>
            <Dashboard v-if="global.page == 'dashboard'"></Dashboard>
            <Course v-if="global.page == 'course'"></Course>
            <Train v-if="global.page == 'train'"></Train>
        </div>
    </div>

    <NotifZone></NotifZone>
</template>
