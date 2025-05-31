import mongoose, { Schema, Document, Model, Types } from 'mongoose';

export interface IDirectMessage extends Document {
  from: Types.ObjectId;
  to: Types.ObjectId;
  content: string;
  attachments?: string[];
  reactions?: { emoji: string; userIds: Types.ObjectId[] }[];
  createdAt: Date;
}

const DirectMessageSchema: Schema<IDirectMessage> = new Schema({
  from: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  to: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  content: { type: String, required: true },
  attachments: [{ type: String }],
  reactions: [{
    emoji: { type: String, required: true },
    userIds: [{ type: Schema.Types.ObjectId, ref: 'User' }],
  }],
  createdAt: { type: Date, default: Date.now },
});

export default (mongoose.models.DirectMessage as Model<IDirectMessage>) || mongoose.model<IDirectMessage>('DirectMessage', DirectMessageSchema); 