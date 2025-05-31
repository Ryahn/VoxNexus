<template>
	<div
		class="flex flex-col w-full h-full pb-6 overflow-auto smallScroleBar bg-nightgray"
	>
		<div
			class="flex items-end justify-center flex-shrink-0 w-full text-xs font-semibold text-gray-500 h-14"
		>
			<h1 class="w-full h-6 pl-4">ONLINE-{{ users.length }}</h1>
		</div>
		<div class="w-full h-10 pl-2" v-for="(user, i) in users" :key="i">
			<div class="flex items-center">
				<img :src="user.avatarUrl || 'https://ui-avatars.com/api/?name=' + user.userName" class="w-8 h-8 rounded-full mr-2 object-cover" alt="Avatar" />
				<div class="flex flex-col">
					<span class="text-green-400">{{ user.userName }}</span>
					<span class="text-xs text-gray-400" v-if="user.status">{{ user.status }}</span>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
type Props = {
	users: {
		avatarUrl?: string
		avatarUrlImg: string
		name: string
		userName: string
		status?: string
		action: {
			name: string
		}
	}[]
}
defineProps<Props>()
</script>

<style scoped>
/* Track */
.smallScroleBar::-webkit-scrollbar-track {
	background: transparent;
}

/* width */
.smallScroleBar::-webkit-scrollbar {
	width: 6px;
}

/* Handle */
.smallScroleBar::-webkit-scrollbar-thumb {
	background: rgb(57, 36, 36);
	border-radius: 10px;
}

/* Handle on hover */
.smallScroleBar::-webkit-scrollbar-thumb:hover {
	background: #555;
}
</style>
