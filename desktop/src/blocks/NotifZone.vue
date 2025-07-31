<script setup lang="ts">
import { NotifType } from '../util';

// The zone dedicated to popup notifications, with style customisation. In pair with justNotify wrapper in util.ts
// This code is based on example of the notiwind npm package
// https://www.npmjs.com/package/notiwind
//
// Note: Closing notification doesnt work for now ... The cursor cannot reach it because of 'pointer-events-none' I guess (from the example snippet)
// but when we remove it, all other elements become impossible to click on.

function bgColorByType(type: NotifType) {
    switch (type) {
        case NotifType.Info:
            return "bg-gray-100"
        case NotifType.Error:
            return "bg-red-100"
        case NotifType.Debug:
            return "bg-yellow-100"
        case NotifType.ServerError:
            return "bg-red-300"
    }
}
</script>

<template>
    <NotificationGroup group="notifs">
        <div class="fixed inset-0 flex items-start justify-end p-6 px-4 py-6 pointer-events-none">
            <div class="w-full max-w-sm">
                <Notification v-slot="{ notifications, close, hovering }"
                    enter="transform ease-out duration-300 transition"
                    enter-from="translate-y-2 opacity-0 sm:translate-y-0 sm:translate-x-4"
                    enter-to="translate-y-0 opacity-100 sm:translate-x-0" leave="transition ease-in duration-500"
                    leave-from="opacity-100" leave-to="opacity-0" move="transition duration-500" move-delay="delay-300">
                    <div class="flex w-full max-w-sm mx-auto mt-4 overflow-hidden rounded-lg shadow-md p-2"
                        @mouseover="hovering(notification.id, true)" @mouseleave="hovering(notification.id, false)"
                        :class="bgColorByType(notification.type)" v-for="notification in notifications"
                        :key="notification.id">
                        <div class="flex">
                            <p class="text-gray-800">{{ notification.text }}</p>
                        </div>
                    </div>
                </Notification>
            </div>
        </div>
    </NotificationGroup>
</template>
