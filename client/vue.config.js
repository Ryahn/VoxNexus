const { defineConfig } = require('@vue/cli-service')

module.exports = defineConfig({
  transpileDependencies: true,
  pluginOptions: {
    electronBuilder: {
      mainProcessWatch: ['src/background.js'],
      mainProcessFile: 'src/background.js',
      rendererProcessFile: 'src/main.js',
      removeDevtoolsExtension: true,
      builderOptions: {
        appId: 'com.voxnexus.app',
        productName: 'VoxNexus',
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
        },
        extraResources: ['./public'],
        asar: false
      }
    }
  }
})
