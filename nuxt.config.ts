import { defineNuxtConfig } from 'nuxt/config'

export default defineNuxtConfig({
	css: ['@/assets/css/tailwind.css'],

	vite: {
		server: {
			allowedHosts: ['voxnexus.test'],
		},
	},

	modules: [
		'@nuxt/ui',
		'@nuxtjs/tailwindcss',
		'@pinia/nuxt',
		
	],
})
