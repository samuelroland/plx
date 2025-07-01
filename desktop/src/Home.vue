<script setup lang="ts">
import { onMounted, Ref, ref } from 'vue';
import { commands, ProjectInfo, Session } from "./bindings";

let props = defineProps<{ sessionDashboardOpen: (session: Session) => void, joinLiveSessionFn: () => void }>()
let course: Ref<ProjectInfo | null> = ref(null)
let projects: Ref<ProjectInfo[]> = ref([])
let sessions: Ref<Session[]> = ref([])

async function loadProjects() {
    projects.value = await commands.getProjects()
    console.log(projects.value)
}

async function loadSessions() {
    sessions.value = await commands.getSessions()
    console.log(projects.value)
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
        const success = await commands.cloneProject(git_url)
        if (success) {
            loadSessions()
        }
    }
}

async function openCourse(path: string) {
    const result = await commands.openProject(path)
    course.value = result as unknown as ProjectInfo
    console.log("course loaded !")
    const result2 = await commands.getSessions()
    if (result2.status == "ok") {
        sessions.value = result2.data
        console.log(sessions.value)
    }
}

async function startSession() {
    const name = prompt("Enter a session name")
    if (name) {
        const success = await commands.startSession(name)
        if (success.status == "ok") {
            props.sessionDashboardOpen(success.data)
        }
    }
}

async function joinSession(session: Session) {
    const success = await commands.joinSession(session)
    if (success) {
        props.sessionDashboardOpen(session)
    }
}


</script>

<template>
    <div class="p-2 sm:mx-5 md:mx-10 lg:mx-10 max-w-[1300px] flex items-center flex-col">
        <img class="w-72 max-w-[80vw]" src="/logo.svg" />
        <div>
            <a target="_blank" href="https://github.com/samuelroland/plx">Git repository</a>
        </div>
        <!-- <h1 class="text-xl md:text-4xl my-5 nice-gradient">Practice programming in a deliberate Learning eXperience </h1> -->

        <div class="flex items-center">
            <h2>Available courses</h2>
            <div class="mt-4"><button @click="cloneCourse">Add course</button></div>
        </div>
        <ol>
            <li @click="openCourse(project.folder)" class="hover:bg-orange-100 cursor-pointer p-2"
                v-for="project in projects">{{ project.name }}
            </li>
        </ol>
        <div class="text-gray-700 italic" v-if="projects.length == 0">No course found...</div>
        <h2>Quick actions</h2>
        <div class="flex">
            <button @click="startSession">Start live session</button>
            <button @click="() => props.joinLiveSessionFn()">Join live session</button>
        </div>
        <div class="text-gray-700 italic" v-if="course == null">Pick a course first</div>
        <div class="text-gray-700 italic" v-if="course != null && sessions.length == 0">No session found, create a new
            one...</div>
        <ol>
            <li @click="joinSession(session)" class="hover:bg-orange-100 cursor-pointer p-2"
                v-for="session in sessions">{{ session.name }} {{ session.group_id }}
            </li>
        </ol>
    </div>
</template>
