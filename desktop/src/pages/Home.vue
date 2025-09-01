<script setup lang="ts">
import { onMounted, Ref, ref } from 'vue';
import { commands, CourseWithConfig } from "../ts/commands.ts";
import { useLiveStore } from '../stores/LiveStore.ts';
import { useTrainStore } from '../stores/TrainStore.ts';
import { useGlobalStore } from '../stores/GlobalStore.ts';
import { Session } from '../ts/shared.ts';
import { justNotify, NotifType } from '../util.ts';
import { onKeyStroke } from '@vueuse/core';

let courses: Ref<CourseWithConfig[]> = ref([])

let selectedCoursePath: Ref<string | null> = ref(null)

let git_https_url: Ref<string> = ref("")
let live_session_name: Ref<string> = ref("")

const live = useLiveStore()
const train = useTrainStore()

async function loadCourses() {
    courses.value = await commands.getLocalCourses()
    console.log(courses.value)
}

onMounted(async () => {
    try {
        loadCourses()
    } catch (error) {
        justNotify(NotifType.Error, error as string)
    }

    // DEBUG: Setup alt+r to reset the client_id as a way to do demo
    // with new clients without using the persisted client_id in localStorage
    onKeyStroke((e) => {
        if (e.key == "r" && e.altKey) {
            const id = live.reset_client_with_non_persisted_client_id()
            justNotify(NotifType.Debug, "New client id generated just for this PLX instance with id " + id, 2000)
            e.preventDefault()
        }
    })
})



async function cloneCourse() {
    if (git_https_url.value && git_https_url.value.trim().length > 0) {
        const success = await commands.cloneCourse(git_https_url.value)
        if (success.status == "ok") {
            justNotify(NotifType.Success, "Successfully cloned the given repository.\nIf that's a valid PLX course, it will be listed below.")
            git_https_url.value = ""
        } else {
            justNotify(NotifType.Error, "Failed to clone the given repository:\n" + success.error)
        }
        loadCourses()
    }
}

async function openCourse(path: string) {
    live.disconnect_if_existing_client()
    live.available_sessions = []; // reset so we don't see some sessions for the previously selected course
    selectedCoursePath.value = path
    let course = courses.value.find(c => c.course.folder == path)
    if (course) {
        live.course = course
    } else {
        justNotify(NotifType.Error, "Course doesn't exist at path " + path)
    }
    live.get_sessions()
}

async function startSession() {
    const name = live_session_name.value
    if (name && name.trim().length > 0) {
        live.start_session(name)
        let course_path = selectedCoursePath.value
        if (course_path)
            train.loadCourse(course_path)
    }
}

async function joinSession(session: Session) {
    live.join_session(session.name)
    let course_path = selectedCoursePath.value
    if (course_path)
        train.loadCourse(course_path)
}

function trainLocally() {
    let course_path = selectedCoursePath.value
    if (course_path)
        train.loadCourse(course_path)
    const global = useGlobalStore()
    global.page = "course";
    // We opened the course locally, we don't care about the server not being reachable
    // TODO: remove that when the session access is refactored
    live.clients_timeout_ids.map((id) => clearInterval(id));
}

async function gitPullAllCourses() {
    await commands.gitPullAllCourses()
    loadCourses()
    justNotify(NotifType.Info, "All courses content should be pulled now")
}

</script>

<template>
    <div class="p-2 sm:mx-5 md:mx-10 lg:mx-10 max-w-[1300px] flex items-center flex-col">
        <img class="w-72 max-w-[80vw]" src="/logo.svg" />

        <h2 class="">Courses</h2>
        <div class="flex items-center w-full mb-5">
            <input v-model="git_https_url" type="text" placeholder="Git HTTPS URL of the course" class="px-1 w-96">
            <button @click="cloneCourse">Add course</button>
            <button v-if="selectedCoursePath != null" @click="trainLocally()">Train {{courses.find(c => c.course.folder
                ==
                selectedCoursePath)?.course.code}} locally</button>
            <button @click="gitPullAllCourses()">Update all</button>
        </div>
        <div class="flex space-x-3">
            <div @click="() => openCourse(pack.course.folder)"
                class="cursor-pointer p-2 border border-orange-500 hover:bg-orange-300/80  rounded-md"
                :class="selectedCoursePath == pack.course.folder ? 'bg-orange-200' : ''" v-for="pack in courses">
                <h3>{{ pack.course.code }}</h3>
                <div>{{ pack.course.name }}</div>
            </div>
            <div v-if="courses.length == 0" class="italic text-gray-600">No course cloned at the moment</div>
        </div>

        <div class="text-gray-700 italic" v-if="courses.length == 0">No course found...</div>
        <div v-if="selectedCoursePath">
            <div class="flex">

                <div class="flex items-center w-full my-5">
                    <input v-model="live_session_name" type="text" placeholder="Name of the session" class="px-1 w-96">
                    <button @click="startSession()">Start live session</button>
                </div>
            </div>
            <div class="text-gray-700 italic" v-if="live.course == null">Pick a course first</div>
            <div class="text-gray-700 italic" v-if="live.course != null && live.available_sessions.length == 0">
                No session running for this course on <strong>{{ live.course.config?.domain }}</strong>, create
                a new one...
            </div>
            <div v-if="live.course != null && live.available_sessions.length > 0">
                <span>Join one of the live session</span>
                <button @click="() => live.get_sessions()">Reload</button>
            </div>
            <ol>
                <li @click="joinSession(session)" class="hover:bg-orange-100 cursor-pointer p-2"
                    v-for="session in live.available_sessions">{{ session.name }}
                </li>
            </ol>
        </div>
    </div>
</template>
