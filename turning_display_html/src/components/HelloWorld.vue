<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";

const date = ref(new Date().toLocaleString());

let intervalId: number;

onMounted(() => {
    console.log("Component has been mounted");
    intervalId = setInterval(() => {
        date.value = new Date().toLocaleString();
    }, 1000);
});

onUnmounted(() => {
    clearInterval(intervalId);
});

</script>

<template>
    <h1>Hello World</h1>
    <p>{{ date }}</p>
</template>