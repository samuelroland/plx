<script setup lang="ts">
import Home from "./Home.vue";
import Join from "./Join.vue";
import Start from "./Start.vue";
import Dashboard from "./Dashboard.vue";
import { Ref, ref } from "vue";
import { Page } from "./types";
import { Session } from "./bindings";

const page: Ref<Page> = ref("home")
const session: Ref<Session | null> = ref(null)

async function openSession(newSession: Session) {
    page.value = 'dashboard'
    session.value = newSession
}
</script>

<template>
    <div class="w-full h-[100vh]">
        <header class="h-7 bg-orange-200 flex px-2 p-1 flex">
            <span class="font-bold flex-1">PLX</span>
            <span v-if="session">{{ session.name }}</span>
        </header>
        <div class="h-[95vh] flex items-center justify-center">
            <Home v-if="page == 'home'" :join-live-session-fn="() => page = 'join'" :sessionDashboardOpen="openSession">
            </Home>
            <Join v-if="page == 'join'"></Join>
            <Start v-if="page == 'start'"></Start>
            <Dashboard v-if="page == 'dashboard'"></Dashboard>
        </div>
    </div>
</template>
