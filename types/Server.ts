export type ChannelType = 'text' | 'voice'

export type Message = {
	id: string
	content: string
	author: string // Use string for user id reference
	created: string
	updated: string
}

export type Category = {
	id: string
	name: string
	order: number
	unRead: number
	channels: string[] // Use string for channel id reference
}

export type Channel = {
	id: string
	name: string
	type: ChannelType
	order: number
	unRead: number
	categoryId: string | null
	messages: Message[]
}

export type Server = {
	id: string
	name: string
	profileImage: string
	order: number
	unRead: number
	channels: Channel[]
	categories: Category[]
} 