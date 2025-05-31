import mongoose, { Schema, Document, Model, Types } from 'mongoose';

export interface IGroupDMMessage extends Document {
  groupId: Types.ObjectId;
  authorId: Types.ObjectId;
  content: string;
  attachments?: string[];
  reactions?: { emoji: string; userIds: Types.ObjectId[] }[];
  createdAt: Date;
  updatedAt: Date;
}

const GroupDMMessageSchema: Schema<IGroupDMMessage> = new Schema({
  groupId: { type: Schema.Types.ObjectId, ref: 'GroupDM', required: true },
  authorId: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  content: { type: String, required: true },
  attachments: [{ type: String }],
  reactions: [{
    emoji: { type: String, required: true },
    userIds: [{ type: Schema.Types.ObjectId, ref: 'User' }],
  }],
  createdAt: { type: Date, default: Date.now },
  updatedAt: { type: Date, default: Date.now }
});

export default (mongoose.models.GroupDMMessage as Model<IGroupDMMessage>) || mongoose.model<IGroupDMMessage>('GroupDMMessage', GroupDMMessageSchema); 