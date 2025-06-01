<template>
	<div class="overflow-hidden w-18 bg-mostnightgray">
		<div class="flex flex-col h-full pt-3 overflow-auto hidescrollbar w-18">
			<!--Direct message dm-->
			<div class="w-full h-12 mb-2">
				<DiscordButton />
			</div>

			<div class="relative flex justify-center flex-shrink-0 w-full mb-2 h-2px">
				<div class="absolute w-8 bg-gray-800 rounded-full h-2px"></div>
			</div>

			<!--servers-->
			<div class="flex justify-center w-full h-auto">
				<div class="w-full">
					<div class="w-full">
						<div v-for="server in servers" :key="server.id" v-if="server && server.id">
							<ServerButton :server="server" />
						</div>
					</div>
				</div>
			</div>

			<!-- Bottom section -->
			<div class="h-auto">
				<slot name="createServerButton"></slot>
				<slot name="exploreButton"></slot>

				<!-- Divide -->
				<div
					class="relative flex justify-center flex-shrink-0 w-full mb-2 h-2px"
				>
					<div class="absolute w-8 bg-gray-800 rounded-full h-2px"></div>
				</div>

				<slot name="downloadButton"></slot>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { useUserStore } from '~/store/user-store'
import { ref, onMounted } from 'vue'
import type { Server } from '~/types/Server'

const userStore = useUserStore()
const servers = ref<Server[]>([])

onMounted(async () => {
	try {
		const res = await $fetch('/api/servers')
		servers.value = res.servers || []
	} catch (e) {
		servers.value = []
		// Optionally show an error or redirect
	}
})
</script>
