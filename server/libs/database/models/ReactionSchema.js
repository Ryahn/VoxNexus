import mongoose from 'mongoose';
import utility from '../../utils';

/**
 * Schema for message reactions
 * Tracks emoji reactions on messages
 */
const ReactionSchema = new mongoose.Schema({
  _id: {
    type: String,
    default: () => utility.generateSnowflake().toString()
  },
  message: {
    type: String,
    ref: 'Message',
    required: true,
    validate: {
      validator: function(v) {
        return mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid message reference'
    }
  },
  user: {
    type: String,
    ref: 'User',
    required: true,
    validate: {
      validator: function(v) {
        return mongoose.Types.ObjectId.isValid(v);
      },
      message: 'Invalid user reference'
    }
  },
  emoji: {
    type: String,
    required: true,
    trim: true
  },
  count: {
    type: Number,
    default: 1
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

// Create compound index for unique reactions per user per message
ReactionSchema.index({ message: 1, user: 1, emoji: 1 }, { unique: true });
ReactionSchema.index({ message: 1, emoji: 1 });
ReactionSchema.index({ user: 1 });
ReactionSchema.index({ createdAt: 1 });

// Update timestamp before saving
ReactionSchema.pre('save', function(next) {
  this.updatedAt = new Date();
  next();
});

// Method to increment reaction count
ReactionSchema.methods.incrementCount = async function() {
  this.count += 1;
  await this.save();
};

// Method to decrement reaction count
ReactionSchema.methods.decrementCount = async function() {
  if (this.count > 0) {
    this.count -= 1;
    await this.save();
  }
};

// Static method to get all reactions for a message
ReactionSchema.statics.getReactionsForMessage = async function(messageId) {
  return this.find({ message: messageId })
    .populate('user', 'username avatar')
    .sort({ createdAt: 1 });
};

// Static method to get all reactions by a user
ReactionSchema.statics.getReactionsByUser = async function(userId) {
  return this.find({ user: userId })
    .populate('message', 'content')
    .sort({ createdAt: -1 });
};

// Static method to remove all reactions from a message
ReactionSchema.statics.removeAllFromMessage = async function(messageId) {
  return this.deleteMany({ message: messageId });
};

// Static method to remove all reactions by a user
ReactionSchema.statics.removeAllByUser = async function(userId) {
  return this.deleteMany({ user: userId });
};

export default mongoose.model('Reaction', ReactionSchema); 