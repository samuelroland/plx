<script setup lang="ts">
import { onMounted } from "vue";
import AnswerShow from "./AnswerShow.vue";
import { useLiveStore } from "./stores/LiveStore.ts";

const live = useLiveStore()
onMounted(() => { })

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
            <div v-for="answer in live.answers.values()">
                <AnswerShow :answer="answer"></AnswerShow>
            </div>
        </div>
    </div>
</template>
