<script setup lang="ts">
import { notify } from 'notiwind'
import { onMounted, ref } from 'vue';
import { useTrainStore } from '../stores/TrainStore';
import { commands, Exo, Skill } from '../ts/commands';
import { onKeyStroke } from "@vueuse/core"
import CodeExo from '../blocks/CodeExo.vue';
import { useGlobalStore } from '../stores/GlobalStore';
import { justNotify, NotifType } from '../util';

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
    if (train.exosSelection && (train.currentSkill()?.exos.length ?? 0) > 0) {
        global.page = "train"
    }
}

// TODO: fix that to only pull the current course !
async function gitPullAllCourses() {
    await commands.gitPullAllCourses()
    train.loadCourse(train.course?.folder ?? "")
    justNotify(NotifType.Info, "Latest course content should have been pulled now")
}

// TODO: remove this when Delibay and PLX have been merged. This is a hotfix to remove the lid in the skill name like "[hab] Introduction" but only keep  "Introduction"
function cleanSkillName(name: string) {
    return name.replace(/\[[a-zA-Z0-9]{3}\]/, "").trim()
}

function totalExosCount() {
    const train = useTrainStore()
    return train.course?.skills.map(s => s.exos.length).reduce((a, b) => a + b, 0) ?? 0
}

</script>

<template>
    <div v-if="train.course" class="h-full w-full px-5">
        <div class="flex items-center">
            <h1 class="flex-1">{{ train.course.name }}</h1>

            <button @click="gitPullAllCourses">Pull updates</button>
            <span title="Run `plx parse` in the course folder to inspect them" v-if="train.errors.length > 0">
                ({{ train.errors.length }} parsing errors)</span>
        </div>

        <div class="flex space-x-3">
            <!-- skills -->
            <div class="flex-1 mr-3">
                <h2 class="flex items-center">
                    <span class="mr-2 flex-1">Skills</span>
                    <span>{{ totalExosCount() }}</span>
                </h2>
                <div v-for="(skill, idx) in train.course.skills"
                    :class="train.selectedSkillIdx == idx ? 'bg-blue-200' : 'hover:bg-blue-50'"
                    class="px-2 text-2xl cursor-pointer"
                    @click="train.selectedSkillIdx = idx; train.exosSelection = false">
                    <div class="flex">
                        <div class="flex-1 line-clamp-1"><span class="mr-3">{{ idx + 1 }}</span>{{
                            cleanSkillName(skill.name) }}
                        </div>
                        <div class="ml-2">{{ skill.exos.length }}</div>
                    </div>
                </div>
            </div>

            <!-- exos -->
            <div class="flex-1">
                <h2>Exos</h2>
                <div v-for="(exo, idx) in train.currentSkill()?.exos"
                    :class="train.exosSelection && train.selectedExoIdx == idx ? 'bg-blue-200' : ''"
                    class="px-2 text-2xl cursor-pointer hover:bg-blue-100"
                    @click="train.selectedExoIdx = idx; train.exosSelection = true" @dblclick="startExo">

                    <div class="flex">
                        <div class="flex-1 flex">
                            <span class="mr-3">{{ (train.selectedSkillIdx + 1) + "." + (idx + 1) }}</span>
                            <span class="line-clamp-1">{{ exo.name }}</span>
                        </div>
                        <span class="hidden md:block ml-4">{{ exo.state }}</span>
                    </div>
                </div>
                <div v-if="train.currentSkill()?.exos.length == 0" class="text-gray-600 italic ">
                    No exo in this skill for now...</div>
            </div>

            <!-- exo preview -->
            <div class="flex-2 hidden lg:block">
                <CodeExo v-if="train.currentExo() != undefined && train.exosSelection" :exo="train.currentExo()">
                </CodeExo>
            </div>
        </div>

    </div>
</template>
