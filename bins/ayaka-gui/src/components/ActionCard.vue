<script lang="ts">
export default {
    props: { ch: String, line: String, sub_line: String }
}
</script>

<template>
    <div class="card">
        <div class="card-header char">
            <h4 class="card-title">{{ ch }}</h4>
        </div>
        <div class="card-body lines">
            <p class="h4 card-text">
                <span v-html="line"></span>
            </p>
            <p class="h4 card-text">
                <span v-html="sub_line"></span>
            </p>
        </div>
    </div>
</template>

<style>
.char {
    height: 3em;
}

.lines {
    height: 8em;
}
</style>
