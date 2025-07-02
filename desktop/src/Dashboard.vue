<script setup lang="ts">
import { onMounted, ref, Ref } from "vue";
import AnswerShow from "./AnswerShow.vue";
import { DEFAULT_LIVE_PORT } from "./ts/commands.ts";
import { CheckStatus, ClientNum, ForwardedFile } from "./ts/bindings.ts";
import { LiveClient } from "./client.ts";

export interface Answer {
    client_num: ClientNum;
    files: Map<string, ForwardedFile>;
    checks_status: CheckStatus[];
}
const answers: Ref<Map<number, Answer>> | null = ref(new Map())


onMounted(() => {
    const client = LiveClient.connect("127.0.0.1", DEFAULT_LIVE_PORT, "super id")
})

</script>

<template>
    <div class="lg:flex w-full p-5 h-full">
        <div class="flex-1 min-w-2/5">
            <h1>Moyenne d'un vecteur</h1>
            <p>Créer une fonction vectorAverage() qui prend en paramètre un vecteur de int et retourne la moyenne des
                valeurs.</p>

            <h3>Checks</h3>

            <h4>Simple vecteur 1,2,3,4</h4>
            <p>Arguments: ["4"]</p>
            <p>Output: The average of first 4 values is 2.5</p>

            <h4>Vecteur vide -> zéro</h4>
            <p>Arguments: ["0"]</p>
            <p>Output: The average of first 0 values is 0</p>
        </div>
        <div class="flex-2 ml-5">
            <h2>Answers</h2>
            {{ answers.values() }}
            <div v-for="answer in answers.values()">
                <AnswerShow :answer="answer"></AnswerShow>
            </div>
        </div>
    </div>
</template>
