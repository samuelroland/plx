<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useTrainStore } from '../stores/TrainStore';
import { commands, Exo, Skill } from '../ts/commands';
import { onKeyStroke } from "@vueuse/core"
import CodeExo from '../blocks/CodeExo.vue';
import { useGlobalStore } from '../stores/GlobalStore';

// Overview of a course, its skills and exos

const train = useTrainStore()
const global = useGlobalStore()

onMounted(() => {
    // Define up and down actions on selection
    onKeyStroke(['j', 'ArrowDown'], (_) => {
        if (train.exosSelection) {
            train.switchExo(1)
        } else {
            train.switchSkill(1)
        }
    })
    onKeyStroke(['k', 'ArrowUp'], (_) => {
        if (train.exosSelection) {
            train.switchExo(-1)
        } else {
            train.switchSkill(-1)
        }
    })
    onKeyStroke(['l', 'ArrowRight'], (_) => {
        train.exosSelection = true
        train.selectedExoIdx = 0
    })
    onKeyStroke(['h', 'ArrowLeft'], (_) => {
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
        startExo()
    })
})

function startExo() {
    if (train.exosSelection) {
        global.page = "train"
    }
}

// TODO: fix that to only pull the current course !
async function gitPullAllCourses() {
    await commands.gitPullAllCourses()
    train.loadCourse(train.course?.folder ?? "")
    alert("Should be pulled now")
}

</script>

<template>
    <div v-if="train.course" class="h-full w-full px-5">
        <div class="flex items-center">
            <h1 class="flex-1">{{ train.course.name }}</h1>

            <button @click="gitPullAllCourses">Pull all</button>
            <span title="Run `plx parse` in the course folder to inspect them" v-if="train.errors.length > 0">({{
                train.errors.length }} parsing errors)</span>
        </div>

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
                    @click="train.selectedExoIdx = idx; train.exosSelection = true" @dblclick="startExo">

                    <div class="flex">
                        <div class="flex-1">
                            <span class="mr-3">{{ (train.selectedSkillIdx + 1) + "." + (idx + 1) }}</span>{{ exo.name }}
                        </div>
                        <span class="hidden md:block">{{ exo.state }}</span>
                    </div>
                </div>
            </div>

            <!-- exo preview -->
            <div class="flex-3 hidden lg:block">
                <CodeExo v-if="train.currentExo() != undefined && train.exosSelection" :exo="train.currentExo()">
                </CodeExo>
            </div>
        </div>

    </div>
</template>
