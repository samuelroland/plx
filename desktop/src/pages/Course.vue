<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useTrainStore } from '../stores/TrainStore';
import { Exo, Skill } from '../ts/commands';
import { onKeyStroke } from "@vueuse/core"
import CodeExo from '../blocks/CodeExo.vue';

// Overview of a course, its skills and exos

const train = useTrainStore()

const selectedSkillIdx = ref(0)
const selectedExoIdx = ref(0)
const exosSelection = ref(false) // skills selection by default, exos selection with right arrow or l key

function currentSkill(): Skill | undefined {
    return train.course?.skills[selectedSkillIdx.value]
}

function currentExo(): Exo | undefined {
    return train.course?.skills[selectedSkillIdx.value].exos[selectedExoIdx.value]
}

function switchExo(increment: number) {
    const length = currentSkill()?.exos.length
    if (length) {
        let newIndex = selectedExoIdx.value
        newIndex += increment
        if (newIndex >= length) {
            newIndex = length - 1
        } else if (newIndex < 0) {
            newIndex = 0
        }
        selectedExoIdx.value = newIndex
    }
}

function switchSkill(increment: number) {
    if (train.course?.skills.length) {
        let newIndex = selectedSkillIdx.value
        newIndex += increment
        if (newIndex >= train.course?.skills.length) {
            newIndex = train.course?.skills.length - 1
        } else if (newIndex < 0) {
            newIndex = 0
        }
        selectedSkillIdx.value = newIndex
    }
}

onMounted(() => {
    // Define up and down actions on selection
    onKeyStroke(['j', 'ArrowDown'], (e) => {
        if (exosSelection.value) {
            switchExo(1)
        } else {
            switchSkill(1)
        }
    })
    onKeyStroke(['k', 'ArrowUp'], (e) => {
        if (exosSelection.value) {
            switchExo(-1)
        } else {
            switchSkill(-1)
        }
    })
    onKeyStroke(['l', 'ArrowRight'], (e) => {
        exosSelection.value = true
        selectedExoIdx.value = 0
    })
    onKeyStroke(['h', 'ArrowLeft'], (e) => {
        exosSelection.value = false
    })
    // TODO: make that a gg not a single g
    onKeyStroke(['g'], (_) => {
        if (exosSelection.value) {
            selectedExoIdx.value = 0
        } else {
            selectedSkillIdx.value = 0
        }
    })
    onKeyStroke(['G'], (_) => {
        if (exosSelection.value) {
            selectedExoIdx.value = (currentSkill()?.exos.length ?? 1) - 1
        } else {
            selectedSkillIdx.value = (train.course?.skills.length ?? 0) - 1
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
                    :class="selectedSkillIdx == idx ? 'bg-blue-200' : 'hover:bg-blue-50'"
                    class="px-2 text-2xl cursor-pointer" @click="selectedSkillIdx = idx; exosSelection = false">
                    <span class="mr-3">{{ idx + 1 }}</span>{{ skill.name }}
                </div>
            </div>

            <!-- exos -->
            <div class="flex-2">
                <h2>Exos</h2>
                <div v-for="(exo, idx) in currentSkill()?.exos"
                    :class="exosSelection && selectedExoIdx == idx ? 'bg-blue-200' : ''"
                    class="px-2 text-2xl cursor-pointer hover:bg-blue-100"
                    @click="selectedExoIdx = idx; exosSelection = true">
                    <div class="flex">
                        <div class="flex-1">
                            <span class="mr-3">{{ (selectedSkillIdx + 1) + "." + (idx + 1) }}</span>{{ exo.name }}
                        </div>
                        <span>{{ exo.state }}</span>
                    </div>
                </div>
            </div>

            <!-- exo preview -->
            <div class="flex-2">
                <CodeExo v-if="currentExo() != undefined && exosSelection" :exo="currentExo()"></CodeExo>
            </div>
        </div>

    </div>
</template>
