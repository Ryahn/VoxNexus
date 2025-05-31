<template>
	<div
		class="relative flex flex-col w-72 p-4 rounded-lg bg-gray-800 shadow-lg hover:shadow-xl transition-all duration-150 text-gray-200 focus-within:ring-2 focus-within:ring-accent"
		tabindex="0"
		role="listitem"
		:aria-label="`Friend card for ${user.username}#${user.tag}`"
	>
		<!-- Avatar and Status -->
		<div class="flex items-center gap-4">
			<div class="relative w-14 h-14">
				<img
					class="w-14 h-14 rounded-full object-cover border-2 border-gray-700"
					:src="user.avatarUrl"
					:alt="`${user.username}'s avatar`"
				/>
				<span
					class="absolute bottom-0 right-0 w-4 h-4 rounded-full border-2 border-gray-800"
					:class="statusColorClass"
					:aria-label="userStatusLabel"
				></span>
			</div>
			<div class="flex flex-col">
				<span class="font-semibold text-lg leading-tight">{{ user.username }}<span class="text-gray-400">{{ user.tag }}</span></span>
				<span class="text-sm text-gray-400 flex items-center gap-1">
					<span v-if="user.statusEmoji">{{ user.statusEmoji }}</span>
					<span>{{ user.statusText }}</span>
				</span>
			</div>
		</div>

		<!-- Bio/About Me -->
		<div v-if="user.bio" class="mt-2 text-sm text-gray-300 italic truncate" :title="user.bio">
			{{ user.bio }}
		</div>

		<!-- Mutual Servers -->
		<div v-if="mutualServers && mutualServers.length" class="mt-3 flex items-center gap-2">
			<span class="text-xs text-gray-400">Mutual Servers:</span>
			<div class="flex items-center gap-1">
				<template v-for="(server, idx) in displayedServers" :key="server.id">
					<img
						:src="server.avatarUrl"
						:alt="server.name"
						class="w-6 h-6 rounded-full border border-gray-700"
						:title="server.name"
					/>
				</template>
				<span v-if="mutualServers.length > 3" class="text-xs text-gray-400 ml-1" :title="allServerNames">
					+{{ mutualServers.length - 3 }}
				</span>
			</div>
		</div>

		<!-- Actions -->
		<div class="mt-4 flex gap-2">
			<button
				class="px-3 py-1 rounded bg-accent text-white hover:bg-accent-dark focus:outline-none focus:ring-2 focus:ring-accent text-sm"
				@click="$emit('message', user)"
				aria-label="Message user"
			>
				Message
			</button>
			<button
				class="px-3 py-1 rounded bg-gray-700 text-gray-200 hover:bg-red-600 hover:text-white focus:outline-none focus:ring-2 focus:ring-red-500 text-sm"
				@click="$emit('remove', user)"
				aria-label="Remove friend"
			>
				Remove
			</button>
			<button
				class="px-3 py-1 rounded bg-gray-700 text-gray-200 hover:bg-gray-500 focus:outline-none focus:ring-2 focus:ring-gray-400 text-sm"
				@click="$emit('block', user)"
				aria-label="Block user"
			>
				Block
			</button>
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface MutualServer {
	id: string
	name: string
	avatarUrl: string
}

interface FriendCardUser {
	id: string
	username: string
	tag: string
	avatarUrl: string
	bio?: string
	status?: 'online' | 'idle' | 'dnd' | 'offline'
	statusEmoji?: string
	statusText?: string
}

const props = defineProps<{
	user: FriendCardUser
	mutualServers?: MutualServer[]
	isOnline?: boolean
}>()

defineEmits<{
	(e: 'message', user: FriendCardUser): void
	(e: 'remove', user: FriendCardUser): void
	(e: 'block', user: FriendCardUser): void
}>()

const statusColorClass = computed(() => {
	switch (props.user.status) {
		case 'online':
			return 'bg-green-500'
		case 'idle':
			return 'bg-yellow-400'
		case 'dnd':
			return 'bg-red-500'
		default:
			return 'bg-gray-500'
	}
})

const userStatusLabel = computed(() => {
	switch (props.user.status) {
		case 'online':
			return 'Online'
		case 'idle':
			return 'Idle'
		case 'dnd':
			return 'Do Not Disturb'
		default:
			return 'Offline'
	}
})

const displayedServers = computed(() => props.mutualServers?.slice(0, 3) || [])
const allServerNames = computed(() => props.mutualServers?.map(s => s.name).join(', ') || '')
</script>

<style scoped>
.bg-accent {
	@apply bg-indigo-600;
}
.bg-accent-dark {
	@apply bg-indigo-700;
}
</style>
