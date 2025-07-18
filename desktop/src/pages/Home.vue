<script setup lang="ts">
import { onMounted, Ref, ref } from 'vue';
import { commands, CourseInfo } from "../ts/commands.ts";
import { useLiveStore } from '../stores/LiveStore.ts';
import { useTrainStore } from '../stores/TrainStore.ts';

let courses: Ref<CourseInfo[]> = ref([])

let selectedCoursePath: Ref<string | null> = ref(null)

const live = useLiveStore()

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
    selectedCoursePath.value = path
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

function trainLocally(course_path: string) {
    const train = useTrainStore()
    train.loadCourse(course_path)
}

async function gitPullAllCourses() {
    await commands.gitPullAllCourses()
    loadCourses()
    alert("Should be pulled now")
}

</script>

<template>
    <div class="p-2 sm:mx-5 md:mx-10 lg:mx-10 max-w-[1300px] flex items-center flex-col">
        <img class="w-72 max-w-[80vw]" src="/logo.svg" />
        <!-- <div> -->
        <!--     <a target="_blank" href="https://github.com/samuelroland/plx">Git repository</a> -->
        <!-- </div> -->
        <!-- <h1 class="text-xl md:text-4xl my-5 nice-gradient">Practice programming in a deliberate Learning eXperience </h1> -->

        <div class="flex items-center w-full">
            <div class="my-6 text-gray-600 flex-1">Select a course to train</div>
            <button @click="gitPullAllCourses">Pull course updates</button>
        </div>
        <div class="flex space-x-3">
            <div @click="() => openCourse(pack.course.folder)"
                class="cursor-pointer p-2 border border-orange-500 rounded-md"
                :class="selectedCoursePath == pack.course.folder ? 'bg-orange-200' : ''" v-for="pack in courses">
                <h3>{{ pack.course.code }}</h3>
                <div>{{ pack.course.name }}</div>
            </div>
            <div v-if="courses.length == 0" class="italic text-gray-600">No course cloned at the moment</div>
        </div>

        <div class="mt-4"><button @click="cloneCourse">Add course</button></div>
        <div class="text-gray-700 italic" v-if="courses.length == 0">No course found...</div>
        <div v-if="selectedCoursePath">
            <div class="flex">
                <button @click="startSession">Start live session</button>
                <button @click="trainLocally(selectedCoursePath)">Train locally</button>
            </div>
            <div class="text-gray-700 italic" v-if="live.course == null">Pick a course first</div>
            <div class="text-gray-700 italic" v-if="live.course != null && live.available_sessions.length == 0">
                No session running for this course, create a new one...
            </div>
            <div v-if="live.course != null && live.available_sessions.length > 0">Join one of the live session:
                <button @click="() => live.get_sessions()">Reload</button>
            </div>
            <ol>
                <li @click="live.join_session(session.name)" class="hover:bg-orange-100 cursor-pointer p-2"
                    v-for="session in live.available_sessions">{{ session.name }}
                </li>
            </ol>
        </div>
    </div>
</template>
