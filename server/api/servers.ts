import type { Server } from '~/types/Server'
import type { H3Event } from 'h3'

const servers: Server[] = [
	{
		id: '1abc',
		name: 'USK',
		profileImage: '',
		unRead: 0,
		order: 0,
		categories: [
			{
				id: '1-newbie',
				name: 'newbie',
				order: 0,
				unRead: 0,
				channels: ['1'],
			},
		],
		channels: [
			{
				id: '2abc',
				name: 'general',
				type: 'text',
				order: 0,
				unRead: 0,
				categoryId: '1-newbie',
				messages: [
					{
						id: '1',
						content: 'Hello World',
						author: 'user-123',
						created: '2021-01-01T00:00:00.000Z',
						updated: '2021-01-01T00:00:00.000Z',
					},
				],
			},

			{
				id: '3abc',
				name: 'general',
				type: 'text',
				order: 0,
				unRead: 0,
				categoryId: null,
				messages: [
					{
						id: 'msg-112',
						content: 'Hello everyone',
						author: 'username#1345',
						created: '2023-01-01T00:00:00.000Z',
						updated: '2023-02-01T00:00:00.000Z',
					},
				],
			},
		],
	},
]

export default defineEventHandler((event: H3Event) => {
	return servers
})
