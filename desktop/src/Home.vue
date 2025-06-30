<script setup lang="ts">
let props = defineProps<{ startLiveSessionFn: () => void, joinLiveSessionFn: () => void }>()
</script>

<template>
    <div class="p-2 sm:mx-5 md:mx-10 lg:mx-10 max-w-[1300px] flex items-center flex-col">
        <img class="w-72 max-w-[80vw]" src="/logo.svg" />
        <div>
            <a target="_blank" href="https://github.com/samuelroland/plx">Git repository</a>
        </div>
        <!-- <h1 class="text-xl md:text-4xl my-5 nice-gradient">Practice programming in a deliberate Learning eXperience </h1> -->

        <h2>Quick actions</h2>
        <div class="flex">
            <button @click="() => props.startLiveSessionFn()">Start live session</button>
            <button @click="() => props.joinLiveSessionFn()">Join live session</button>
        </div>
    </div>
</template>
