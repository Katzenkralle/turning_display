<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";

let buttons = ref([
    { x: -10, y: 10, color: "red" },
    { x: 10, y: 10, color: "green" },
    { x: 10, y: -10, color: "blue" },
    { x: -10, y: -10, color: "yellow" },
]);

const renderReady = ref(false);

onMounted(() => {
    renderReady.value = true;
    const observer = new ResizeObserver(() => {
        renderReady.value = false;
        setTimeout(() => {
            renderReady.value = true;
        }, 0);
    });

    const innerCircle = document.getElementById("innerCircle");
    if (innerCircle) {
        observer.observe(innerCircle);
    }

    onUnmounted(() => {
        if (innerCircle) {
            observer.unobserve(innerCircle);
        }
    });
});

const getFurthestPoint = computed(() => {
    let max = 0;
    for (let btn of buttons.value) {
        const distance = Math.sqrt(btn.x ** 2 + btn.y ** 2);
        if (distance > max) {
            max = distance;
        }
    }
    return max+2;
});



const getPossition = (x: number, y: number) => {
    const innerCircle = document.getElementById("innerCircle");
    console.log(innerCircle.offsetWidth/2);
    
    if (!innerCircle){
        console.error("innerCircle not found");
        return "";
    } 
    
    const center = innerCircle.offsetWidth / 2;
    const scale = (innerCircle.offsetWidth / 2) / getFurthestPoint.value;
    x = x * scale;
    y = y * scale;

    // Adjust x and y to be relative to the center
    x = center + x;
    y = center - y;

    console.log(x, y);  
    const angle = Math.atan2(x - center, y - center) * (180 / Math.PI);

    // ofset needet for acuret display
    x -= 15;
    y -= 15;
    return {
        left: x + "px",
        top: y + "px",
        transform: "rotate(" + angle + "deg)",
    };
};

</script>

<template>
    <div class="border-4">
        <div id="innerCircle">
            <div class="vertical-line"></div>
            <div class="horizontal-line"></div>

            <div v-if="renderReady" v-for="btn in buttons" :key="btn.color">
                <button class="lightButton" :style="{ backgroundColor: btn.color, ...getPossition(btn.x, btn.y) }"></button>
            </div>
        </div>
    </div>
</template>

<style scoped>
#innerCircle {
  position: relative;
  border: 5px solid red;
  aspect-ratio: 1 / 1;
  border-radius: 50%;
  margin: 0 auto;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: black;
}

/* Blue cross lines */
.vertical-line,
.horizontal-line {
  position: absolute;
  background-color: rgb(77, 77, 96);
}

.vertical-line {
  width: 5px;
  height: 100%;
}

.horizontal-line {
  height: 5px;
  width: 100%;
}

.lightButton {
  position: absolute;
  width: 20px;
  height: 20px;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  display: flex;
  justify-content: center;
  align-items: center;
  font-size: 16px;
  font-weight: bold;
}
</style>