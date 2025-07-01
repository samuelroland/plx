<script setup lang="ts">

let props = defineProps<{ code: string }>()

</script>

<template>
    <code>
        <pre class="border border-orange-400 rounded-sm p-2">{{ props.code.trim() }}</pre>
    </code>
</template>
