<template>
	<div class="flex h-screen w-screen bg-[var(--color-bg-main)]">
		<!-- Dark mode toggle button -->
		<button
			class="fixed top-4 right-4 z-50 p-2 rounded-md bg-[var(--color-bg-secondary)] text-[var(--color-text-main)] shadow-lg border border-[var(--color-border)] hover:bg-[var(--color-bg-tertiary)]"
			@click="toggleDarkMode"
			aria-label="Toggle dark mode"
		>
			<svg v-if="isDark" class="w-6 h-6" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" d="M12 3v1m0 16v1m8.66-13.66l-.71.71M4.05 19.07l-.71.71M21 12h-1M4 12H3m16.66 5.66l-.71-.71M4.05 4.93l-.71-.71M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
			</svg>
			<svg v-else class="w-6 h-6" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" d="M21 12.79A9 9 0 1111.21 3a7 7 0 109.79 9.79z" />
			</svg>
		</button>
		<!-- Server Sidebar (icons) -->
		<ServerSidebar v-if="isHydrated && userStore.user">
			<template #createServerButton>
				<CreateServerButton />
			</template>
			<template #exploreButton>
				<ExploreButton />
			</template>
			<template #downloadButton>
				<DownloadButton />
			</template>
		</ServerSidebar>
		<!-- Group DM Sidebar -->
		<group-dm-sidebar v-if="isHydrated && userStore.user" />
		<!-- Main content -->
		<main class="flex-1 flex flex-col min-w-0 bg-nightgraylighter">
			<!-- Top bar (optional, move toolbar here if needed) -->
			<div class="relative flex items-center w-full h-12 px-2 border-b border-black min-w-34">
				<!-- Add your toolbar or header here if needed -->
			</div>
			<div class="flex-1 min-h-0 overflow-auto">
				<NuxtPage v-if="isHydrated" />
			</div>
		</main>
		<ToastContainer />
	</div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useUserStore } from '~/store/user-store'
import ToastContainer from '~/components/ToastContainer.vue'
import ServerSidebar from '~/components/server/sidebar.vue'
import CreateServerButton from '~/components/create-server-button.vue'
import ExploreButton from '~/components/explore-button.vue'
import DownloadButton from '~/components/download-button.vue'

const userStore = useUserStore()
const creatingServer = ref(false)
const isDark = ref(true)
const isHydrated = ref(false)

const onClickOutsideServer = () => {
	creatingServer.value = false
}

function toggleDarkMode() {
	isDark.value = !isDark.value
	if (isDark.value) {
		document.documentElement.setAttribute('data-theme', 'dark')
	} else {
		document.documentElement.removeAttribute('data-theme')
	}
}

onMounted(async () => {
	if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
		isDark.value = true
		document.documentElement.setAttribute('data-theme', 'dark')
	}
	await userStore.fetchProfile()
	isHydrated.value = true
})
</script>
