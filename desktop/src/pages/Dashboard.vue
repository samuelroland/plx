<script setup lang="ts">
import AnswerShow from "../blocks/AnswerShow.vue";
import { LiveSessionStep, useLiveStore } from "../stores/LiveStore.ts";
import ExosSelection from "../blocks/ExosSelection.vue";
import CodeExo from "../blocks/CodeExo.vue";
import { useTrainStore } from "../stores/TrainStore.ts";

const live = useLiveStore()

</script>

<template>
    <div class="lg:flex w-full px-5 h-full">
        <!-- exos selection step -->
        <ExosSelection v-if="live.live_session_step == LiveSessionStep.EXOS_SELECTION"></ExosSelection>

        <div v-if="live.live_session_step == LiveSessionStep.RUNNING" class="flex-1 min-w-2/5">
            <CodeExo :exo="live.currentLiveExo()"></CodeExo>
            <div>
                <h2>Stats</h2>
                <div class="flex space-x-4">
                    <h4>Followers: {{ live.stats?.followers_count }}</h4>
                    <h4>Leaders: {{ live.stats?.leaders_count }}</h4>
                </div>
                <h3>{{ live.live_current_exo_index + 1 }}/{{ live.live_exos_ids.length }}</h3>
                <button @click="live.changeLiveExoIndex(-1)">Previous exo</button>
                <button @click="live.changeLiveExoIndex(1)">Next exo</button>
            </div>
        </div>
        <div class="flex-2 ml-5 overflow-hidden max-h-full">
            <h2>Answers</h2>
            <div class="overflow-auto h-full pb-20">
                <span v-if="live.answers.size == 0" class="text-gray-800 italic">
                    No code or check result has been sent yet
                </span>
                <div v-for="answer in live.answers.values()">
                    <AnswerShow :answer="answer"></AnswerShow>
                </div>
            </div>
        </div>
    </div>
</template>
