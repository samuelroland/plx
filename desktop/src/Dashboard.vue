<script setup lang="ts">
import { onMounted, ref, Ref } from "vue";
import Code from "./Code.vue";
import { commands, ProjectInfo, Session } from "./bindings";

let course: Ref<Exo | null> = ref(null)

type Answer = string
const answers: Ref<Map<number, Answer>> | null = ref(new Map())

// async function getExo() {
//
// }
let codes = [
    `
double vectorAverage(const vector<int> & vec) {
    long sum = 0;
    for (auto v: vec) {
        sum += v;
    }
    return (double) sum / vec.size();
}
`,
    `
double vectorAverage(const vector<int> &vec) {
    return (double) accumulate(vec.begin(), vec.end(), 0) / vec.size();
}
`,
    `double vectorAverage(const vector<int> &vec) {
    long sum = 0;
    for (
    return (double) sum / vec.size();
}
`,
    `double vectorAverage(const vector<int> &vec) {
    long sum = 0;
    for (int i = 0; i < vec.size(); i++) {
        sum += vec[i];
    }
    if (vec.size() == 0) return 0;
    return (double) sum / vec.size();
}
`
]

onMounted(() => {
    codes.forEach((code, idx) => {
        answers.value.set(idx, code)
    })
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
            <div v-for="answer in answers">
                <div class="my-2"><Code :code="answer[1]"></Code></div>
            </div>
        </div>
    </div>
</template>
