<template>
  <div class="grid grid-cols-3 gap-4 mb-6">
    <div class="bg-blue-100 p-4 rounded">
      <h4 class="font-bold">Total Transactions</h4>
      <p>{{ parsedLogs.length }}</p>
    </div>
    <div class="bg-green-100 p-4 rounded">
      <h4 class="font-bold">Unique Programs</h4>
      <p>{{ uniqueProgramsCount }}</p>
    </div>
    <div class="bg-purple-100 p-4 rounded">
      <h4 class="font-bold">Last Update</h4>
      <p>{{ lastUpdateTime }}</p>
    </div>
  </div>
</template>

<script>
export default {
  props: ['parsedLogs', 'uniqueProgramsCount', 'lastUpdateTime']
};
</script>