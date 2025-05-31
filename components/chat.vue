<template>
	<div class="relative flex flex-1 min-w-0 pr-2 sm:pr-12 mt-2 sm:mt-4 hover:bg-[var(--color-bg-secondary)] group py-2 sm:py-0">
		<div
			class="flex items-center justify-center flex-shrink-0 w-10 h-10 sm:w-12 sm:h-12 mx-2 sm:mx-3 select-none"
		>
			<div class="w-8 h-8 sm:w-10 sm:h-10 overflow-hidden rounded-full">
				<img :src="chat.author.avatarUrl || 'https://ui-avatars.com/api/?name=' + chat.author.userName" alt="" srcset="" />
			</div>
		</div>
		<div class="flex-shrink inline-block w-full h-auto min-w-0">
			<div class="w-full h-8 flex items-center">
				<span class="text-[var(--color-text-main)] text-sm sm:text-base">{{ chat.author.userName }}</span>
				<span class="text-xs text-[var(--color-text-muted)] ml-2">{{ chat.date }}</span>
				<span v-if="chat.edited" class="text-xs text-[var(--color-text-secondary)] ml-2">(edited)</span>
			</div>
			<div
				:id="id"
				@keydown.enter.exact="onSubmit"
				@keydown.esc="onCancelEditing"
				:contenteditable="editing"
				:class="{
					'bg-[var(--color-bg-input)] pr-4 pl-2 messageinput rounded-lg overflow-x-hidden overflow-y-auto':
						editing,
				}"
				class="text-base break-all outline-none text-[var(--color-text-main)] md:break-words"
			>
				<!--message-->
				{{ compiledMarkdown }}
			</div>

			<!-- Attachments -->
			<div v-if="chat.attachments && chat.attachments.length" class="mt-2 flex flex-wrap gap-2">
				<div v-for="(file, idx) in chat.attachments" :key="idx" class="w-full sm:w-auto">
					<img v-if="isImage(file)" :src="file" class="max-h-32 w-full sm:w-auto rounded-lg border border-[var(--color-border)] object-contain" />
					<a v-else :href="file" target="_blank" class="text-[var(--color-accent)] underline break-all">Attachment</a>
				</div>
			</div>

			<!-- Reactions Bar -->
			<div v-if="chat.reactions && chat.reactions.length" class="flex gap-1 mt-2 flex-wrap">
				<button
					v-for="reaction in chat.reactions"
					:key="reaction.emoji"
					@click="handleReaction(reaction.emoji)"
					@mouseenter="(e) => showTooltip(e, reaction)"
					@mouseleave="hideTooltip"
					@keyup.enter="handleReaction(reaction.emoji)"
					@keyup.space="handleReaction(reaction.emoji)"
					:tabindex="0"
					role="button"
					:aria-label="`React with ${reaction.emoji}, ${reaction.userIds.length} users`"
					:aria-pressed="hasReacted(reaction)"
					:class="{'bg-[var(--color-accent)] text-white': hasReacted(reaction), 'scale-110 transition-transform duration-200': animatedReactions[reaction.emoji]}"
					class="flex items-center px-2 py-1 rounded-full bg-[var(--color-bg-tertiary)] hover:bg-[var(--color-bg-secondary)] text-sm relative min-w-[40px] min-h-[32px] border border-[var(--color-border)] focus:ring-2 focus:ring-[var(--color-accent)]"
				>
					<span class="text-lg">{{ reaction.emoji }}</span>
					<span class="ml-1">{{ reaction.userIds.length }}</span>
				</button>
				<!-- Add Reaction Button (UI only) -->
				<button
					@click="showEmojiPicker = !showEmojiPicker"
					@keyup.enter="showEmojiPicker = !showEmojiPicker"
					@keyup.space="showEmojiPicker = !showEmojiPicker"
					tabindex="0"
					role="button"
					aria-label="Add reaction"
					class="ml-1 px-2 py-1 rounded-full bg-[var(--color-bg-secondary)] hover:bg-[var(--color-bg-tertiary)] text-sm min-w-[40px] min-h-[32px] border border-[var(--color-border)] focus:ring-2 focus:ring-[var(--color-accent)]"
				>+</button>
				<!-- Tooltip -->
				<div
					v-if="tooltipReaction && tooltipUsers.length"
					:style="{ position: 'fixed', left: tooltipX + 'px', top: (tooltipY + 20) + 'px', zIndex: 1000 }"
					class="px-3 py-2 rounded bg-[var(--color-bg-overlay)] text-[var(--color-text-main)] text-xs shadow-lg border border-[var(--color-border)] pointer-events-none animate-fade-in"
					aria-live="polite"
				>
					<span v-for="(user, idx) in tooltipUsers" :key="user">
						{{ user }}<span v-if="idx < tooltipUsers.length - 1">, </span>
					</span>
					<span v-if="tooltipUsers.length === 0">No reactions</span>
				</div>
			</div>
			<!-- Emoji Picker (placeholder) -->
			<div v-if="showEmojiPicker" class="absolute z-10 mt-2 bg-[var(--color-bg-overlay)] border border-[var(--color-border)] rounded shadow-lg p-2 flex flex-wrap gap-1" tabindex="0" @keydown.esc="showEmojiPicker = false">
				<button
					v-for="emoji in emojiList"
					:key="emoji"
					@click="addReaction(emoji)"
					@keyup.enter="addReaction(emoji)"
					@keyup.space="addReaction(emoji)"
					tabindex="0"
					role="button"
					:aria-label="`React with ${emoji}`"
					class="text-xl hover:bg-[var(--color-bg-secondary)] rounded p-1 min-w-[40px] min-h-[40px] focus:ring-2 focus:ring-[var(--color-accent)]"
				>{{ emoji }}</button>
			</div>

			<div v-if="editing" class="text-sm text-[var(--color-text-muted)]">
				escape to
				<a @click="onCancelEditing" role="button" class="text-[var(--color-accent)]">cancel</a>
				• enter to
				<a @click="onSubmit" role="button" class="text-[var(--color-accent)]">save</a>
			</div>
		</div>

		<div class="absolute top-0 right-0 invisible group-hover:visible">
			<div class="flex items-center text-[var(--color-text-secondary)] rounded-md bg-[var(--color-bg-secondary)] border border-[var(--color-border)]">
				<!-- File upload button -->
				<button @click="triggerFileInput" class="flex items-center justify-center w-8 h-8 btn hover:bg-[var(--color-bg-tertiary)]">
					<svg width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
				</button>
				<input ref="fileInput" type="file" class="hidden" @change="onFileChange" />
				<!-- Edit button -->
				<button @click="onEditing" class="flex items-center justify-center w-8 h-8 btn hover:bg-[var(--color-bg-tertiary)]">
					<svg class="" aria-hidden="false" width="24" height="24" viewBox="0 0 24 24">
						<path fill-rule="evenodd" clip-rule="evenodd" d="M19.293 9.83l.648-.647a3.628 3.628 0 000-5.124 3.628 3.628 0 00-5.124 0l-.647.648 5.123 5.123zm-6.397-3.853L5.185 13.69l5.124 5.122 7.711-7.714-5.124-5.122zM4.12 20.97l4.64-1.159-4.572-4.572-1.16 4.64a.9.9 0 001.092 1.091z" fill="currentColor"/>
					</svg>
				</button>
				<!-- Delete button -->
				<button @click="onDelete" class="flex items-center justify-center w-8 h-8 btn hover:bg-[var(--color-danger)]">
					<svg width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m5 0V4a2 2 0 0 1 2-2h0a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>
				</button>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { marked } from 'marked'
import { ref, onMounted, computed, watch } from 'vue'
import { useMessageActions } from '@/composables/useMessageActions'
import { useUserStore } from '@/store/user-store'

type Reaction = { emoji: string; userIds: string[] }
type Chat = {
	author: {
		userName: string
		avatarUrl?: string
		avatar: string
		avatarUrlImg: string
	}
	date: string
	value: string
	attachments?: string[]
	reactions?: Reaction[]
	edited?: boolean
	id: string
	channelId?: string
	groupId?: string
	userId?: string
}

type Props = {
	chat: Chat
	id: string
	currentUserId?: string
	contextType: 'channel' | 'group-dm' | 'dm'
}

const { chat, id, currentUserId, contextType } = defineProps<Props>()

const editing = ref(false)
const showEmojiPicker = ref(false)
const emojiList = ['😀','😂','😍','😎','👍','🎉','🔥','😢','😡','🙏','🚀']

const md = ref(chat.value)
const { reactToMessage, editMessage, deleteMessage, uploadAttachment, isLoading, error } = useMessageActions()

const userStore = useUserStore()

// Tooltip state
const tooltipReaction = ref<string | null>(null)
const tooltipX = ref(0)
const tooltipY = ref(0)
const tooltipUsers = ref<string[]>([])

function getUsernames(userIds: string[]): string[] {
	return userIds
		.map(id => userStore.allUsers.find(u => u.id === id)?.username || 'Unknown')
		.filter(Boolean)
}

function showTooltip(event: MouseEvent, reaction: Reaction) {
	tooltipReaction.value = reaction.emoji
	tooltipUsers.value = getUsernames(reaction.userIds)
	tooltipX.value = event.clientX
	tooltipY.value = event.clientY
}
function hideTooltip() {
	tooltipReaction.value = null
	tooltipUsers.value = []
}

// Animation state for reactions
const animatedReactions = ref<{ [emoji: string]: boolean }>({})

watch(
	() => chat.reactions?.map(r => `${r.emoji}:${r.userIds.length}`).join(','),
	() => {
		if (!chat.reactions) return
		chat.reactions.forEach(r => {
			animatedReactions.value[r.emoji] = true
			setTimeout(() => (animatedReactions.value[r.emoji] = false), 300)
		})
	}
)

const onSubmit = async () => {
	editing.value = false
	md.value = textArea.value
	chat.value = md.value
	try {
		await editMessage({
			type: contextType,
			ids: {
				channelId: chat.channelId,
				groupId: chat.groupId,
				userId: chat.userId,
				messageId: chat.id
			},
			content: md.value,
			attachments: chat.attachments
		})
		chat.edited = true
	} catch (e) {
		// error.value is set by composable
	}
}

const onCancelEditing = () => {
	editing.value = false
	textArea.value = chat.value
}

const onEditing = () => {
	editing.value = true
	textArea.value = md.value
}

const textArea = ref<HTMLTextAreaElement>()
onMounted(() => {
	textArea.value = document.getElementById(`${id}`) as HTMLTextAreaElement
})

const compiledMarkdown = marked(md.value || '')

function isImage(file: string) {
	return /\.(jpg|jpeg|png|gif|webp)$/i.test(file)
}

function hasReacted(reaction: Reaction) {
	return currentUserId && reaction.userIds.includes(currentUserId)
}

async function handleReaction(emoji: string) {
	if (!currentUserId) return
	const reaction = chat.reactions?.find(r => r.emoji === emoji)
	const hasUser = reaction?.userIds.includes(currentUserId)
	try {
		await reactToMessage({
			type: contextType,
			ids: {
				channelId: chat.channelId,
				groupId: chat.groupId,
				userId: chat.userId,
				messageId: chat.id
			},
			emoji,
			action: hasUser ? 'remove' : 'add'
		})
		// Optimistically update UI (in real app, update from socket event)
		if (!chat.reactions) chat.reactions = []
		if (hasUser) {
			reaction!.userIds = reaction!.userIds.filter(id => id !== currentUserId)
			if (reaction!.userIds.length === 0) {
				chat.reactions = chat.reactions.filter(r => r.emoji !== emoji)
			}
		} else {
			if (reaction) {
				reaction.userIds.push(currentUserId)
			} else {
				chat.reactions.push({ emoji, userIds: [currentUserId] })
			}
		}
	} catch (e) {
		// error.value is set by composable
	}
}

function addReaction(emoji: string) {
	showEmojiPicker.value = false
	handleReaction(emoji)
}

// File upload logic
const fileInput = ref<HTMLInputElement | null>(null)
function triggerFileInput() {
	fileInput.value?.click()
}
async function onFileChange(e: Event) {
	const files = (e.target as HTMLInputElement).files
	if (!files || !files.length) return
	const file = files[0]
	try {
		const res = await uploadAttachment({
			type: contextType,
			ids: {
				channelId: chat.channelId,
				userId: chat.userId
			},
			file
		})
		if (res.attachmentUrl) {
			if (!chat.attachments) chat.attachments = []
			chat.attachments.push(res.attachmentUrl)
			await editMessage({
				type: contextType,
				ids: {
					channelId: chat.channelId,
					groupId: chat.groupId,
					userId: chat.userId,
					messageId: chat.id
				},
				content: chat.value,
				attachments: chat.attachments
			})
		}
	} catch (e) {
		// error.value is set by composable
	}
}

// Delete logic
const emit = defineEmits(['delete'])
async function onDelete() {
	try {
		await deleteMessage({
			type: contextType,
			ids: {
				channelId: chat.channelId,
				groupId: chat.groupId,
				userId: chat.userId,
				messageId: chat.id
			}
		})
		emit('delete', chat.id)
	} catch (e) {
		// error.value is set by composable
	}
}
</script>

<style scoped>
@keyframes fade-in {
	from { opacity: 0; transform: translateY(10px); }
	to { opacity: 1; transform: translateY(0); }
}
.animate-fade-in {
	animation: fade-in 0.2s ease;
}
@media (max-width: 640px) {
	.rounded-lg { border-radius: 0 !important; }
	.pr-12 { padding-right: 0 !important; }
	.mt-4 { margin-top: 0.5rem !important; }
}
</style>
