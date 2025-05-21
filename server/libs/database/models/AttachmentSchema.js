import mongoose from 'mongoose';
import utility from '../../utils';

const AttachmentSchema = new mongoose.Schema({
  _id: {
    type: String,
    default: () => utility.generateSnowflake().toString()
  },
  url: {
    type: String,
    required: true,
    validate: {
      validator: function(v) {
        return !v || /^(https?:\/\/|\/)[^\s]+$/.test(v);
      },
      message: 'URL must be a valid URL or file path'
    }
  },
  type: {
    type: String,
    enum: ['image', 'video', 'audio', 'file'],
    default: 'file'
  },
  size: {
    type: Number,
    required: true
  },
  conversationType: {
    type: String,
    enum: ['server', 'dm', 'group'],
    required: true
  },
  channel: {
    type: String,
    ref: 'Channel'
  },
  server: {
    type: String,
    ref: 'Server'
  },
  createdAt: {
    type: Date,
    default: Date.now
  },
  updatedAt: {
    type: Date,
    default: Date.now
  }
}, {
  timestamps: true
});

// Add validation to ensure proper fields are set based on conversationType
AttachmentSchema.pre('save', function(next) {
  if (this.conversationType === 'server') {
    if (!this.channel || !this.server) {
      next(new Error('Channel and server are required for server attachments'));
    }
  } else if (this.conversationType === 'dm' || this.conversationType === 'group') {
    if (this.server) {
      next(new Error('Server should not be set for DM or group attachments'));
    }
  }
  next();
});

AttachmentSchema.index({ type: 1, size: 1 });
AttachmentSchema.index({ url: 1, server: 1 });
AttachmentSchema.index({ createdAt: 1 });
AttachmentSchema.index({ conversationType: 1 });

export default mongoose.model('Attachment', AttachmentSchema);