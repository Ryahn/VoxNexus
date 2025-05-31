import mongoose, { Schema, Document, Model, Types } from 'mongoose';

export interface IChannelMessage extends Document {
  channelId: Types.ObjectId;
  serverId: Types.ObjectId;
  authorId: Types.ObjectId;
  content: string;
  attachments?: string[];
  reactions?: { emoji: string; userIds: Types.ObjectId[] }[];
  createdAt: Date;
  updatedAt: Date;
}

const ChannelMessageSchema: Schema<IChannelMessage> = new Schema({
  channelId: { type: Schema.Types.ObjectId, ref: 'Channel', required: true },
  serverId: { type: Schema.Types.ObjectId, ref: 'Server', required: true },
  authorId: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  content: { type: String, required: true },
  attachments: [{ type: String }],
  reactions: [{
    emoji: { type: String, required: true },
    userIds: [{ type: Schema.Types.ObjectId, ref: 'User' }],
  }],
  createdAt: { type: Date, default: Date.now },
  updatedAt: { type: Date, default: Date.now },
});

export default (mongoose.models.ChannelMessage as Model<IChannelMessage>) || mongoose.model<IChannelMessage>('ChannelMessage', ChannelMessageSchema); 