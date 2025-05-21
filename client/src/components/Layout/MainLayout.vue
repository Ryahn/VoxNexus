<template>
  <div class="h-screen flex bg-[#36393f]">
    <!-- Server List Sidebar -->
    <div class="flex flex-col bg-[#202225] w-[72px] h-full">
      <ServerList />
    </div>
    <!-- Channel List Sidebar with User Bar at Bottom -->
    <ChannelList>
      <template #user-profile>
        <UserProfile />
      </template>
    </ChannelList>
    <!-- Main Content Area -->
    <div class="flex-1 bg-[#36393f]">
      <slot></slot>
    </div>
  </div>
</template>

<script setup>
import ServerList from '../Server/ServerList.vue'
import ChannelList from '../Server/ChannelList.vue'
import UserProfile from '../User/UserProfile.vue'
</script> 