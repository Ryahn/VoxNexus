import type { BaseAdapter } from './types'
import LocalAdapter from './adapters/local'
// import S3Adapter from './adapters/s3' // Add as needed
// import FTPAdapter from './adapters/ftp'
// import SFTPAdapter from './adapters/sftp'

function createAdapter(): BaseAdapter {
  const storageType = (process.env.STORAGE || 'local').toLowerCase()
  switch (storageType) {
    // case 's3': return new S3Adapter()
    // case 'ftp': return new FTPAdapter()
    // case 'sftp': return new SFTPAdapter()
    case 'local':
    default:
      return new LocalAdapter()
  }
}

const adapter = createAdapter()

const AttachmentService: BaseAdapter = {
  upload: (...args) => adapter.upload(...args),
  delete: (...args) => adapter.delete(...args),
  get: (...args) => adapter.get(...args),
  uploadUserAvatar: (...args) => adapter.uploadUserAvatar(...args),
  uploadUserBanner: (...args) => adapter.uploadUserBanner(...args),
  uploadServerIcon: (...args) => adapter.uploadServerIcon(...args),
  uploadServerBanner: (...args) => adapter.uploadServerBanner(...args),
  uploadChannelIcon: (...args) => adapter.uploadChannelIcon(...args),
  uploadChannelAttachment: (...args) => adapter.uploadChannelAttachment(...args),
  uploadGroupChatAttachment: (...args) => adapter.uploadGroupChatAttachment(...args),
  getUserFiles: (...args) => adapter.getUserFiles(...args),
  getServerFiles: (...args) => adapter.getServerFiles(...args),
  getChannelFiles: (...args) => adapter.getChannelFiles(...args),
  getGroupChatFiles: (...args) => adapter.getGroupChatFiles(...args),
}

export default AttachmentService 