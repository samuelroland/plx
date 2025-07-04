<script setup lang="ts">
import { CheckStatus, ExoCheckResult } from './ts/bindings';
import Code from './Code.vue';

const props = defineProps<{ check: ExoCheckResult }>()

import { ref } from 'vue';

import { autoUpdate, useFloating } from '@floating-ui/vue';
const reference = ref(null);
const floating = ref(null);
const { floatingStyles } = useFloating(reference, floating, {
    placement: 'top-end',
    whileElementsMounted: autoUpdate,
});
const visible = ref(false)
console.log(floatingStyles)

function returnTailwindBackgroundForCheckStatus(state: CheckStatus) {
    switch (state.type) {
        case 'BuildFailed': return "bg-red-200"
        case 'Passed': return "bg-green-200"
        case 'CheckFailed': return "bg-orange-200"
        case 'RunFailed': return "bg-purple-200"
    }
}
</script>

<template>
    <div ref="reference" class="mx-2 text-base p-1 rounded-sm cursor-pointer " @mouseover="visible = true"
        @mouseout="visible = false" :class="returnTailwindBackgroundForCheckStatus(check.state)">
        {{ "C" + props.check.index }}
    </div>
    <div v-if="props.check.state.content != undefined"
        class="absolute z-50 border border-blue-400 rounded-md text-base shadow-md/50" ref="floating"
        :class="visible ? '' : 'hidden'" :style="floatingStyles">
        <Code :code="props.check.state.content"></Code>
    </div>

</template>
