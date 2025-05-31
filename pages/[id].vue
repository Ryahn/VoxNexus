<template>
	<div
		class="relative flex w-screen h-screen overflow-hidden bg-black select-none"
	>
		<!-- Hamburger for mobile -->
		<button
			class="fixed top-4 left-4 z-50 p-2 rounded-md bg-gray-900 text-white shadow-lg lg:hidden"
			@click="sidebarOpen = true"
			aria-label="Open sidebar"
		>
			<svg class="w-6 h-6" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" d="M4 6h16M4 12h16M4 18h16" />
			</svg>
		</button>
		<!-- Sidebar overlay for mobile -->
		<transition name="fade">
			<div
				v-if="sidebarOpen"
				class="fixed inset-0 z-40 bg-black bg-opacity-60 lg:hidden"
				@click="sidebarOpen = false"
				aria-label="Close sidebar overlay"
			></div>
		</transition>
		<aside
			:class="[
				'fixed z-50 top-0 left-0 h-full w-64 bg-nightgray shadow-lg transform transition-transform duration-200 lg:static lg:translate-x-0',
				sidebarOpen ? 'translate-x-0' : '-translate-x-full',
				'lg:relative lg:flex'
			]"
			tabindex="-1"
			aria-label="Sidebar"
		>
			<div class="flex flex-col h-full">
				<div class="flex items-center justify-between h-16 px-4 lg:hidden">
					<span class="text-lg font-bold text-white">Channels</span>
					<button @click="sidebarOpen = false" aria-label="Close sidebar" class="p-2 rounded-md text-gray-400 hover:text-white hover:bg-gray-800">
						<svg class="w-6 h-6" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
						</svg>
					</button>
				</div>
				<ServerSidebar class="flex-1" />
			</div>
		</aside>
		<!-- Main content -->
		<div
			class="flex flex-col flex-1 min-w-0 overflow-hidden bg-green-400 mainsection lg:ml-64"
		>
			<!--Main section nav, sidebar-->

			<div
				class="relative flex items-center flex-shrink-0 w-auto h-12 px-2 border-b border-gray-900 bg-nightgraylighter"
			>
				<div class="flex items-center w-full h-6 overflow-hidden">
					<div class="w-6 h-6 mx-2 text-gray-600">
						<svg width="24" height="24" class="">
							<path
								fill="currentColor"
								fill-rule="evenodd"
								clip-rule="evenodd"
								d="M5.887 21a.5.5 0 01-.493-.587L6 17H2.595a.5.5 0 01-.492-.586l.175-1A.5.5 0 012.77 15h3.58l1.06-6H4.005a.5.5 0 01-.492-.586l.175-1A.5.5 0 014.18 7h3.58l.637-3.587A.5.5 0 018.889 3h.984a.5.5 0 01.493.587L9.76 7h6l.637-3.587A.5.5 0 0116.889 3h.984a.5.5 0 01.493.587L17.76 7h3.405a.5.5 0 01.492.586l-.175 1A.5.5 0 0120.99 9h-3.58l-1.06 6h3.405a.5.5 0 01.492.586l-.175 1a.5.5 0 01-.492.414H16l-.637 3.587a.5.5 0 01-.492.413h-.984a.5.5 0 01-.493-.587L14 17H8l-.637 3.587a.5.5 0 01-.492.413h-.984zM9.41 9l-1.06 6h6l1.06-6h-6z"
							/>
						</svg>
					</div>
					<h2 class="w-auto h-6 mr-2 font-bold tracking-tighter text-gray-300">
						{{ currentServer?.name || 'Server' }}
					</h2>
					<div class="w-px h-6 mx-2 bg-gray-700"></div>
					<div
						class="w-auto h-4 ml-2 text-sm font-semibold leading-none tracking-tighter text-gray-500 truncate"
					>
						Channel: {{ currentChannel?.name || 'Select a channel' }}
					</div>
				</div>

				<div
					class="flex items-center justify-between flex-shrink-0 w-auto h-6 overflow-hidden text-gray-500 bg-nightgraylighter"
				>
					<div class="flex items-center mx-2">
						<button @click="muteServer = !muteServer" class="btn">
							<svg
								v-if="muteServer"
								class=""
								aria-hidden="false"
								width="24"
								height="24"
							>
								<path
									class="text-red-500"
									fill="currentColor"
									d="M21.178 1.707l1.414 1.414L4.121 21.593l-1.414-1.415L21.178 1.707z"
								/>
								<path
									fill="currentColor"
									d="M18 10.528L10.529 18H21v-1a3 3 0 01-3-3v-3.472zM8.957 19.572L9.529 19h5.916c-.693 1.19-1.97 2-3.445 2a3.96 3.96 0 01-3.043-1.428zM12 3c1.417 0 2.71.5 3.734 1.321l-9.736 9.737.001-.03L6 14V9a6 6 0 016-6z"
								/>
							</svg>
							<svg
								v-else
								class=""
								aria-hidden="false"
								fill="none"
								width="24"
								height="24"
							>
								<path
									fill="currentColor"
									fill-rule="evenodd"
									clip-rule="evenodd"
									d="M18 9v5a3 3 0 003 3v1H3v-1a3 3 0 003-3V9a6 6 0 1112 0zm-6 12c-1.476 0-2.752-.81-3.445-2h6.89c-.693 1.19-1.97 2-3.445 2z"
								/>
							</svg>
						</button>
					</div>
					<div class="flex items-center mx-2">
						<button class="btn">
							<svg class="w-6 h-6" aria-hidden="false">
								<path
									fill="currentColor"
									d="M22 12l-9.899-9.899-1.415 1.413 1.415 1.415-4.95 4.949v.002L5.736 8.465 4.322 9.88l4.243 4.242-5.657 5.656 1.414 1.414 5.657-5.656 4.243 4.242 1.414-1.414-1.414-1.414L19.171 12h.001l1.414 1.414L22 12z"
								/>
							</svg>
						</button>
					</div>

					<div class="flex items-center mx-2">
						<button @click="showMembers = !showMembers" class="btn">
							<svg class="w-6 h-6" aria-hidden="false">
								<path
									fill="currentColor"
									fill-rule="evenodd"
									clip-rule="evenodd"
									d="M14 8.006c0 2.205-1.794 4-4 4-2.205 0-4-1.795-4-4s1.794-4 4-4 4 1.795 4 4zm-12 11c0-3.533 3.29-6 8-6 4.711 0 8 2.467 8 6v1H2v-1z"
								/>
								<path
									fill="currentColor"
									fill-rule="evenodd"
									clip-rule="evenodd"
									d="M14 8.006c0 2.205-1.794 4-4 4-2.205 0-4-1.795-4-4s1.794-4 4-4 4 1.795 4 4zm-12 11c0-3.533 3.29-6 8-6 4.711 0 8 2.467 8 6v1H2v-1z"
								/>
								<path
									fill="currentColor"
									d="M20 20.006h2v-1c0-2.563-1.73-4.565-4.479-5.47 1.541 1.377 2.48 3.27 2.48 5.47v1zM14.883 11.908A4.007 4.007 0 0018 8.006a4.006 4.006 0 00-3.503-3.97A5.977 5.977 0 0116 8.007a5.974 5.974 0 01-1.362 3.804c.082.032.164.064.245.098z"
								/>
							</svg>
						</button>
					</div>

					<div
						class="relative flex items-center w-auto h-6 px-1 mx-2 text-sm bg-gray-900 rounded-sm"
					>
						<div class="flex items-center w-full h-6">
							<input
								:class="{ 'w-52': searchFocused || query }"
								class="h-5 font-semibold duration-300 transform bg-transparent outline-none w-28"
								v-model="query"
								@focus="searchFocuse"
								@focusout="searchFocused = false"
								type="text"
								placeholder="Search"
							/>
						</div>
						<svg
							v-if="query.length"
							@click="deleteQuery"
							class="w-6 h-6 transform scale-75 cursor-pointer"
							aria-hidden="false"
						>
							<path
								fill="currentColor"
								d="M18.4 4L12 10.4 5.6 4 4 5.6l6.4 6.4L4 18.4 5.6 20l6.4-6.4 6.4 6.4 1.6-1.6-6.4-6.4L20 5.6 18.4 4z"
							/>
						</svg>
						<svg v-else class="w-6 h-6 transform scale-75" aria-hidden="false">
							<path
								fill="currentColor"
								d="M21.707 20.293L16.314 14.9A7.928 7.928 0 0018 10a7.945 7.945 0 00-2.344-5.656A7.94 7.94 0 0010 2a7.94 7.94 0 00-5.656 2.344A7.945 7.945 0 002 10c0 2.137.833 4.146 2.344 5.656A7.94 7.94 0 0010 18a7.922 7.922 0 004.9-1.686l5.393 5.392 1.414-1.413zM10 16a5.959 5.959 0 01-4.242-1.757A5.958 5.958 0 014 10c0-1.602.624-3.109 1.758-4.242A5.956 5.956 0 0110 4c1.603 0 3.109.624 4.242 1.758A5.957 5.957 0 0116 10c0 1.603-.624 3.11-1.758 4.243A5.959 5.959 0 0110 16z"
							/>
						</svg>
					</div>

					<div class="flex items-center mx-2">
						<button class="btn">
							<svg class="w-6 h-6" aria-hidden="false" fill="none">
								<path
									d="M19 3H4.99c-1.11 0-1.98.89-1.98 2L3 19c0 1.1.88 2 1.99 2H19c1.1 0 2-.9 2-2V5a2 2 0 00-2-2zm0 12h-4c0 1.66-1.35 3-3 3s-3-1.34-3-3H4.99V5H19v10z"
									fill="currentColor"
								/>
							</svg>
						</button>
					</div>

					<a
						class="flex items-center px-1 mx-1"
						href="https://support.discord.com/hc/en-us"
						target="_blank"
					>
						<div class="flex items-center">
							<button class="btn">
								<svg class="w-6 h-6" aria-hidden="false">
									<path
										fill="currentColor"
										d="M12 2C6.486 2 2 6.487 2 12c0 5.515 4.486 10 10 10s10-4.485 10-10c0-5.513-4.486-10-10-10zm0 16.25a1.25 1.25 0 110-2.5 1.25 1.25 0 010 2.5zm1-4.375V15h-2v-3h1a2 2 0 10-2-2H8c0-2.205 1.795-4 4-4s4 1.795 4 4a4.01 4.01 0 01-3 3.875z"
									/>
								</svg>
							</button>
						</div>
					</a>
				</div>
			</div>

			<div class="flex flex-1 min-h-0">
				<!--chat section-->
				<div class="flex flex-col flex-1 select-text">
					<div class="w-full h-1 -mt-1"></div>

					<div
						class="relative flex flex-col flex-1 min-w-0 min-h-0 overflow-hidden font-sans text-lg break-normal bg-nightgraylighter"
					>
						<div id="chatspace" class="flex-1 pb-5 overflow-auto">
							<div class="flex flex-col flex-1 px-2 sm:px-4 md:px-8">
								<div v-for="(msg, i) in chatMessages" :key="i" class="mb-2">
									<div class="flex items-center space-x-2">
										<span class="font-bold text-green-400">{{ msg.username }}</span>
										<span class="text-xs text-gray-500">{{ msg.createdAt }}</span>
									</div>
									<div class="relative">
										<div v-if="editingMessageId === msg._id">
											<input v-model="editInput" class="rounded bg-gray-700 text-white px-2 py-1 w-full" />
											<div class="flex space-x-2 mt-1">
												<button @click="() => confirmEdit(msg)" class="px-2 py-1 bg-green-600 text-white rounded">Save</button>
												<button @click="cancelEdit" class="px-2 py-1 bg-gray-600 text-white rounded">Cancel</button>
											</div>
										</div>
										<div v-else>
											<div class="text-white">{{ msg.content }}</div>
											<div class="flex items-center space-x-1 mt-1">
												<span
													v-for="reaction in msg.reactions || []"
													:key="reaction.emoji"
													class="inline-flex items-center px-2 py-0.5 rounded-full text-xs cursor-pointer relative"
													:class="[
														'bg-gray-700',
														reaction.userIds.includes(userStore.user.id) ? 'ring-2 ring-yellow-400 bg-yellow-100 text-yellow-900' : '',
														animatedReactions[`${i}:${reaction.emoji}`] ? 'scale-110 transition-transform duration-200' : ''
													]"
													@click="handleReact(msg, reaction.emoji)"
													@mouseenter="(e) => showTooltip(e, reaction)"
													@mouseleave="hideTooltip"
													@keyup.enter="handleReact(msg, reaction.emoji)"
													@keyup.space="handleReact(msg, reaction.emoji)"
													tabindex="0"
													role="button"
													:aria-label="`React with ${reaction.emoji}, ${reaction.userIds.length} users`"
													:aria-pressed="reaction.userIds.includes(userStore.user.id)"
												>
													{{ reaction.emoji }} {{ reaction.userIds.length }}
												</span>
												<!-- Tooltip -->
												<div
													v-if="tooltipReaction && tooltipUsers.length && msg.reactions && msg.reactions.some(r => r.emoji === tooltipReaction)"
													:style="{ position: 'fixed', left: tooltipX + 'px', top: (tooltipY + 20) + 'px', zIndex: 1000 }"
													class="px-3 py-2 rounded bg-gray-900 text-gray-100 text-xs shadow-lg border border-gray-700 pointer-events-none animate-fade-in"
													aria-live="polite"
												>
													<span v-for="(user, idx) in tooltipUsers" :key="user">
														{{ user }}<span v-if="idx < tooltipUsers.length - 1">, </span>
													</span>
													<span v-if="tooltipUsers.length === 0">No reactions</span>
												</div>
												<button
													@click="showEmojiPicker = msg._id"
													@keyup.enter="showEmojiPicker = msg._id"
													@keyup.space="showEmojiPicker = msg._id"
													tabindex="0"
													role="button"
													aria-label="Add reaction"
													class="ml-1 text-yellow-400 hover:text-yellow-300"
												>😊</button>
												<div v-if="showEmojiPicker === msg._id" class="absolute z-10 bg-gray-800 border border-gray-700 rounded p-2 mt-1" tabindex="0" @keydown.esc="showEmojiPicker = null">
													<button @click="() => { handleReact(msg, '👍'); showEmojiPicker = null; }" @keyup.enter="() => { handleReact(msg, '👍'); showEmojiPicker = null; }" @keyup.space="() => { handleReact(msg, '👍'); showEmojiPicker = null; }" tabindex="0" role="button" aria-label="React with 👍" class="text-lg">👍</button>
													<button @click="() => { handleReact(msg, '😂'); showEmojiPicker = null; }" @keyup.enter="() => { handleReact(msg, '😂'); showEmojiPicker = null; }" @keyup.space="() => { handleReact(msg, '😂'); showEmojiPicker = null; }" tabindex="0" role="button" aria-label="React with 😂" class="text-lg">😂</button>
													<button @click="() => { handleReact(msg, '❤️'); showEmojiPicker = null; }" @keyup.enter="() => { handleReact(msg, '❤️'); showEmojiPicker = null; }" @keyup.space="() => { handleReact(msg, '❤️'); showEmojiPicker = null; }" tabindex="0" role="button" aria-label="React with ❤️" class="text-lg">❤️</button>
													<button @click="showEmojiPicker = null" @keyup.enter="showEmojiPicker = null" @keyup.space="showEmojiPicker = null" tabindex="0" role="button" aria-label="Close emoji picker" class="ml-2 text-gray-400">×</button>
												</div>
											</div>
										</div>
										<div v-if="msg.userId === userStore.user.id && editingMessageId !== msg._id" class="flex space-x-1 mt-1">
											<button @click="() => startEdit(msg)" class="text-xs text-blue-200 hover:text-blue-400">Edit</button>
											<button @click="() => handleDelete(msg)" class="text-xs text-red-300 hover:text-red-500">Delete</button>
										</div>
									</div>
								</div>
							</div>
						</div>

						<!--Input form-->
						<form @submit.prevent="handleSend" class="flex items-center p-2 sm:p-4 bg-nightgraylighter sticky bottom-0 z-20">
							<input v-model="input" class="flex-1 p-2 rounded bg-gray-700 text-white mr-2" placeholder="Type a message..." />
							<button type="submit" class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700">Send</button>
						</form>
					</div>
				</div>

				<!--show members section-->
				<div
					:class="[showMembers ? 'w-60' : 'w-0', 'h-auto hidden xl:block']"
				>
					<MembersBar :users="users" />
				</div>
			</div>
		</div>

		<!--popup components-->
		<div
			v-if="creatingServer"
			class="absolute flex items-center justify-center w-screen h-screen bg-black bg-opacity-90"
		>
			<CreateServer @click-outside="onClickOutsideServerComponent" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useUserStore } from '~/store/user-store'
import { useServerStore } from '~/store/server-store'
import { useChatStore } from '~/store/chat-store'
import { usePresence } from '~/composables/usePresence'
import { useChat } from '~/composables/useChat'
import type {
	ChannelMessageEditPayload,
	ChannelMessageDeletePayload,
	ChannelMessageReactionPayload
} from '~/types/socket-events';

const route = useRoute()
const userStore = useUserStore()
const serverStore = useServerStore()
const chatStore = useChatStore()

const input = ref('')

const serverId = computed(() => route.params.id as string)
const currentServer = computed(() => serverStore.servers.find(s => s.id === serverId.value))
const currentChannel = computed(() => currentServer.value?.channels[0] || null)
const currentChannelId = computed(() => currentChannel.value?.id || '')

const choosingEmojiOrGif = ref(false)
const creatingServer = ref(false)
const showMembers = ref(true)
const searchFocused = ref(false)
const query = ref('')
const gategoryBtnHover = ref(false)
const muteServer = ref(false)

const editingMessageId = ref<string | null>(null)
const editInput = ref('')
const showEmojiPicker = ref<string | null>(null)

// Tooltip state
const tooltipReaction = ref<string | null>(null)
const tooltipX = ref(0)
const tooltipY = ref(0)
const tooltipUsers = ref<string[]>([])

// Animation state for reactions
const animatedReactions = ref<{ [key: string]: boolean }>({})

const sidebarOpen = ref(false)

const onClickOutsideServerComponent = () => {
	creatingServer.value = false
}

const deleteQuery = () => {
	query.value = ''
}
const searchFocuse = () => {
	searchFocused.value = true
}

// Fetch servers and channels on mount
onMounted(async () => {
	if (userStore.token) {
		await serverStore.fetchServers(userStore.token)
		if (serverId.value) {
			await serverStore.fetchChannels(serverId.value, userStore.token)
		}
	}
})

// Real-time chat composable
const { messages, sendMessage, editMessage, deleteMessage, reactToMessage, leaveChannel } = useChat(userStore.token, currentChannelId.value)
const chatMessages = computed(() => messages.value.get(currentChannelId.value) || [])

// Real-time presence composable
const { onlineUserIds } = usePresence(userStore.token)

function handleSend() {
	if (!input.value.trim() || !userStore.user) return
	sendMessage(input.value, userStore.user.id, userStore.user.username)
	input.value = ''
}

function selectChannel(channelId: string) {
	serverStore.setCurrentChannel(channelId)
	// Optionally, re-initialize chat composable for new channel
}

function startEdit(msg: any) {
	editingMessageId.value = msg._id
	editInput.value = msg.content
}
function cancelEdit() {
	editingMessageId.value = null
	editInput.value = ''
}
function confirmEdit(msg: any) {
	if (!editInput.value.trim()) return
	const payload: ChannelMessageEditPayload = {
		channelId: msg.channelId,
		messageId: msg._id,
		content: editInput.value,
		updatedAt: new Date().toISOString(),
	}
	editMessage(payload)
	cancelEdit()
}
function handleDelete(msg: any) {
	const payload: ChannelMessageDeletePayload = {
		channelId: msg.channelId,
		messageId: msg._id,
	}
	deleteMessage(payload)
}
function handleReact(msg: any, emoji: string) {
	const userId = userStore.user.id;
	const reaction = (msg.reactions || []).find((r: any) => r.emoji === emoji);
	const hasReacted = reaction && reaction.userIds.includes(userId);
	const payload: ChannelMessageReactionPayload = {
		channelId: msg.channelId,
		messageId: msg._id,
		emoji,
		userId,
		action: hasReacted ? 'remove' : 'add',
	};
	reactToMessage(payload);
}

function getUsernames(userIds: string[]): string[] {
	return userIds
		.map(id => userStore.allUsers.find(u => u.id === id)?.username || 'Unknown')
		.filter(Boolean)
}

function showTooltip(event: MouseEvent, reaction: any) {
	tooltipReaction.value = reaction.emoji
	tooltipUsers.value = getUsernames(reaction.userIds)
	tooltipX.value = event.clientX
	tooltipY.value = event.clientY
}

function hideTooltip() {
	tooltipReaction.value = null
	tooltipUsers.value = []
}

watch(
	() => chatMessages.value.map((m: any, i: number) => (m.reactions || []).map((r: any) => `${i}:${r.emoji}:${r.userIds.length}`).join(',')).join(','),
	() => {
		chatMessages.value.forEach((msg: any, i: number) => {
			(msg.reactions || []).forEach((r: any) => {
				animatedReactions.value[`${i}:${r.emoji}`] = true
				setTimeout(() => (animatedReactions.value[`${i}:${r.emoji}`] = false), 300)
			})
		})
	}
)
</script>

<style scoped>
@keyframes fade-in {
	from { opacity: 0; transform: translateY(10px); }
	to { opacity: 1; transform: translateY(0); }
}
.animate-fade-in {
	animation: fade-in 0.2s ease;
}
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
@media (max-width: 1024px) {
	.mainsection { margin-left: 0 !important; }
}
</style>
