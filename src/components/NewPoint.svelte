<script lang="ts">

    type Props = { pointData: PointData }

    let { pointData } : Props = $props() 
</script>


<div class="flex gap-4 m-4 text-xs items-center border-y">
    <div class="h-1 bg-blue-500 min-w-1 rounded"></div>
    <p>{pointData.mag}</p>
    <p>{pointData.flynn_region}</p>
    <p>{pointData.lastupdate}</p>
</div>

