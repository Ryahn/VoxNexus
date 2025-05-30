const { defineConfig } = require('@vue/cli-service')

module.exports = defineConfig({
  transpileDependencies: true,
  // Set the entry point to main.ts
  pages: {
    index: {
      entry: 'src/main.ts',
      title: 'VoxNexus'
    }
  },
  // Add performance optimization settings
  configureWebpack: {
    performance: {
      hints: false
    },
    optimization: {
      splitChunks: {
        minSize: 10000,
        maxSize: 250000
      }
    }
  },
  // Configure asset handling
  chainWebpack: config => {
    // Configure image handling
    config.module
      .rule('images')
      .test(/\.(png|jpe?g|gif|webp|avif|svg)$/i)
      .use('url-loader')
      .loader('url-loader')
      .options({
        limit: 4096,
        fallback: {
          loader: 'file-loader',
          options: {
            name: 'img/[name].[hash:8].[ext]'
          }
        }
      })
      .end()

    // Disable thread-loader for TypeScript
    config.module
      .rule('ts')
      .use('ts-loader')
      .tap(options => ({
        ...options,
        transpileOnly: true,
        appendTsSuffixTo: [/\.vue$/]
      }))
  }
})
