<template>
  <div class="message-input">
    <textarea
      v-model="message"
      @keydown.enter.prevent="handleSubmit"
      placeholder="Type a message..."
      class="w-full p-2 rounded-lg border border-gray-300 focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
    />
    <button
      @click="handleSubmit"
      class="mt-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
    >
      Send
    </button>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'
import type { PropType } from 'vue'

interface MessageInputProps {
  channelId: string
  onSend: (message: string) => Promise<void>
}

export default defineComponent({
  name: 'MessageInput',
  props: {
    channelId: {
      type: String as PropType<string>,
      required: true
    },
    onSend: {
      type: Function as PropType<(message: string) => Promise<void>>,
      required: true
    }
  },
  setup(props) {
    const message = ref('')

    const handleSubmit = async () => {
      if (!message.value.trim()) return
      
      try {
        await props.onSend(message.value)
        message.value = ''
      } catch (error) {
        console.error('Failed to send message:', error)
      }
    }

    return {
      message,
      handleSubmit
    }
  }
})
</script> 