import { defineStore } from 'pinia'

export const useProfileDrawerStore = defineStore('profileDrawer', {
  state: () => ({
    isOpen: false as boolean,
    user: null as any | null
  }),
  actions: {
    openDrawer(user: any) {
      this.user = user
      this.isOpen = true
    },
    closeDrawer() {
      this.isOpen = false
    }
  }
}) 