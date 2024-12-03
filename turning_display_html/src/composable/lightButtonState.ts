import { ref, onMounted, onUnmounted, computed, type Ref } from "vue";

onMounted(() => {
    let LightButtonController = new LightButtonState();
})

export class LightButtonState {
    private buttons: Ref<{ x: number, y: number, color: string }[]>
    private selection: Ref<number[]>
    constructor() {
        this.buttons = ref([
            { x: -10, y: 10, color: "red" },
            { x: 10, y: 10, color: "green" },
            { x: 10, y: -10, color: "blue" },
            { x: -10, y: -10, color: "yellow" },
        ])
        this.selection = ref([])   
    }

    private fetchButtons(): { x: number, y: number, color: string }[] {
        return this.buttons.value;
    }
    
    /**
     * toggleSelection
     */
    public toggleSelection(id: number): void {
        this.selection.value.includes(id) ?
        this.selection.value.splice(this.selection.value.indexOf(id), 1) 
        :
        this.selection.value.push(id);
    }

    public changeColor(color: string): void {
    this.selection.value.forEach((id) => {
        this.buttons.value[id].color = color
    })
    }

    public sendUpdate(): String {
        return JSON.stringify(this.selection.value);
    }
    
}