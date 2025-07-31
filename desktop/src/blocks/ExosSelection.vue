<script setup lang="ts">
import { LiveSessionStep, useLiveStore } from '../stores/LiveStore';
import { useTrainStore } from '../stores/TrainStore';
import { Exo } from '../ts/commands';

const live = useLiveStore()
const train = useTrainStore()

function toggleExo(exo: Exo) {
    const id = exo.folder
    if (live.live_exos_ids.includes(id)) {
        live.live_exos_ids = live.live_exos_ids.filter(el => el != id)
        live.live_exos_map.delete(id)
    } else {
        live.live_exos_ids.push(id)
        live.live_exos_map.set(id, exo)
    }
}

function getIndexSelectedExoIndexOrNothing(course_path: string) {
    const idx = live.live_exos_ids.findIndex(e => e == course_path)
    return idx === -1 ? "" : idx + 1
}

</script>


<template>
    <div class="w-max">
        <h1>Exos selection</h1>
        Selected {{ live.live_exos_ids.length }} exos
        <!-- skills -->
        <div v-for="(skill, idx) in train.course?.skills" class="px-2 text-2xl cursor-pointer">
            <!-- exos -->
            <div class="flex-2">
                <h3>{{ skill.name }}</h3>
                <div v-for="(exo, idx) in skill.exos"
                    :class="live.live_exos_ids.includes(exo.folder) ? 'bg-blue-200' : ''"
                    class="flex px-2 text-2xl cursor-pointer hover:bg-blue-100" @click="toggleExo(exo)">
                    <span class="w-8 block">
                        {{ getIndexSelectedExoIndexOrNothing(exo.folder) }}
                    </span>
                    <!-- <span class="mr-3">{{ (idx + 1) }}</span> -->
                    {{ exo.name }}
                </div>
                <div class="text-gray-600 text-base italic" v-if="skill.exos.length == 0">No exo here</div>
            </div>
        </div>

        <button @click="live.startTraining">Start training with first exo !</button>
    </div>
</template>
