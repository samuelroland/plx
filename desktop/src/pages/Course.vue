<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useTrainStore } from '../stores/TrainStore';
import { Exo, Skill } from '../ts/commands';
import { onKeyStroke } from "@vueuse/core"
import CodeExo from '../blocks/CodeExo.vue';
import { useGlobalStore } from '../stores/GlobalStore';

// Overview of a course, its skills and exos

const train = useTrainStore()
const global = useGlobalStore()

onMounted(() => {
    // Define up and down actions on selection
    onKeyStroke(['j', 'ArrowDown'], (e) => {
        if (train.exosSelection) {
            train.switchExo(1)
        } else {
            train.switchSkill(1)
        }
    })
    onKeyStroke(['k', 'ArrowUp'], (e) => {
        if (train.exosSelection) {
            train.switchExo(-1)
        } else {
            train.switchSkill(-1)
        }
    })
    onKeyStroke(['l', 'ArrowRight'], (e) => {
        train.exosSelection = true
        train.selectedExoIdx = 0
    })
    onKeyStroke(['h', 'ArrowLeft'], (e) => {
        train.exosSelection = false
    })
    // TODO: make that a gg not a single g
    onKeyStroke(['g'], (_) => {
        if (train.exosSelection) {
            train.selectedExoIdx = 0
        } else {
            train.selectedSkillIdx = 0
        }
    })
    onKeyStroke(['G'], (_) => {
        if (train.exosSelection) {
            train.selectedExoIdx = (train.currentSkill()?.exos.length ?? 1) - 1
        } else {
            train.selectedSkillIdx = (train.course?.skills.length ?? 0) - 1
        }
    })
    onKeyStroke(['Enter'], (_) => {
        if (train.exosSelection) {
            global.page = "train"
        }
    })
})

</script>

<template>
    <div v-if="train.course" class="h-full w-full px-5">
        <h1>{{ train.course.name }}</h1>

        <div class="flex space-x-3">
            <!-- skills -->
            <div class="flex-1">
                <h2>Skills</h2>
                <div v-for="(skill, idx) in train.course.skills"
                    :class="train.selectedSkillIdx == idx ? 'bg-blue-200' : 'hover:bg-blue-50'"
                    class="px-2 text-2xl cursor-pointer"
                    @click="train.selectedSkillIdx = idx; train.exosSelection = false">
                    <span class="mr-3">{{ idx + 1 }}</span>{{ skill.name }}
                </div>
            </div>

            <!-- exos -->
            <div class="flex-2">
                <h2>Exos</h2>
                <div v-for="(exo, idx) in train.currentSkill()?.exos"
                    :class="train.exosSelection && train.selectedExoIdx == idx ? 'bg-blue-200' : ''"
                    class="px-2 text-2xl cursor-pointer hover:bg-blue-100"
                    @click="train.selectedExoIdx = idx; train.exosSelection = true">
                    <div class="flex">
                        <div class="flex-1">
                            <span class="mr-3">{{ (train.selectedSkillIdx + 1) + "." + (idx + 1) }}</span>{{ exo.name }}
                        </div>
                        <span>{{ exo.state }}</span>
                    </div>
                </div>
            </div>

            <!-- exo preview -->
            <div class="flex-3">
                <CodeExo v-if="train.currentExo() != undefined && train.exosSelection" :exo="train.currentExo()">
                </CodeExo>
            </div>
        </div>

    </div>
</template>
