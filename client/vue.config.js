const { defineConfig } = require('@vue/cli-service')

module.exports = defineConfig({
  transpileDependencies: true,
  pluginOptions: {
    electronBuilder: {
      builderOptions: {
        appId: 'com.discordclone.app',
        productName: 'Discord Clone',
        win: {
          target: ['nsis'],
          icon: 'public/icon.ico'
        },
        linux: {
          target: ['AppImage', 'deb'],
          icon: 'public/icon.png'
        },
        mac: {
          target: ['dmg'],
          icon: 'public/icon.icns'
        }
      }
    }
  }
})
