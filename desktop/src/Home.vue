<script setup lang="ts">
import { onMounted, Ref, ref } from 'vue';
import { commands, CourseInfo } from "./ts/commands.ts";
import { useLiveStore } from './stores/LiveStore.ts';
import { useGlobalStore } from './stores/GlobalStore.ts';
import { Session } from './ts/bindings.ts';

let courses: Ref<CourseInfo[]> = ref([])

let selectedCourse: Ref<string | null> = ref(null)

const live = useLiveStore()
const global = useGlobalStore()

async function loadProjects() {
    courses.value = await commands.getLocalCourses()
    console.log(courses.value)
}

onMounted(async () => {
    try {
        loadProjects()
    } catch (error) {
        alert(error)
    }
})

async function cloneCourse() {
    const git_url = prompt("Enter a course Git URL")
    if (git_url) {
        const success = await commands.cloneCourse(git_url)
        if (success) {
            live.get_sessions()
        }
    }
}

async function openCourse(path: string) {
    live.available_sessions = []; // reset so we don't see some sessions for the previously selected course
    selectedCourse.value = path
    let course = courses.value.find(c => c.folder == path)
    if (course) {
        live.course = course
    } else {
        alert("Course doesnt exist at path " + path)
    }
    live.get_sessions()
}

async function startSession() {
    const name = prompt("Enter a session name")
    if (name && name.trim().length > 0) {
        live.start_session(name)
    }
}



</script>

<template>
    <div class="p-2 sm:mx-5 md:mx-10 lg:mx-10 max-w-[1300px] flex items-center flex-col">
        <img class="w-72 max-w-[80vw]" src="/logo.svg" />
        <!-- <div> -->
        <!--     <a target="_blank" href="https://github.com/samuelroland/plx">Git repository</a> -->
        <!-- </div> -->
        <!-- <h1 class="text-xl md:text-4xl my-5 nice-gradient">Practice programming in a deliberate Learning eXperience </h1> -->

        <div class="flex items-center">
            <h2>Available courses</h2>
            <div class="mt-4"><button @click="cloneCourse">Add course</button></div>
        </div>
        <div class="flex">
            <div @click="() => openCourse(project.folder)" class="cursor-pointer p-2"
                :class="selectedCourse == project.folder ? 'bg-orange-200' : ''" v-for="project in courses">
                {{ project.name }}
            </div>
        </div>
        <div class="text-gray-700 italic" v-if="courses.length == 0">No course found...</div>
        <div v-if="selectedCourse">
            <div class="flex">
                <button @click="startSession">Start live session</button>
            </div>
            <div class="text-gray-700 italic" v-if="live.course == null">Pick a course first</div>
            <div class="text-gray-700 italic" v-if="live.course != null && live.available_sessions.length == 0">
                No session running for this course, create a new one...
            </div>
            <ol>
                <li @click="live.join_session(session.name)" class="hover:bg-orange-100 cursor-pointer p-2"
                    v-for="session in live.available_sessions">{{ session.name }}
                </li>
            </ol>
        </div>
    </div>
</template>
