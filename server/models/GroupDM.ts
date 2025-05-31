import mongoose, { Schema, Document, Model, Types } from 'mongoose';

export interface IGroupDM extends Document {
  name?: string;
  ownerId: Types.ObjectId;
  memberIds: Types.ObjectId[];
  avatarUrl?: string;
  lastMessageAt?: Date;
  createdAt: Date;
}

const GroupDMSchema: Schema<IGroupDM> = new Schema({
  name: { type: String, trim: true },
  ownerId: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  memberIds: {
    type: [{ type: Schema.Types.ObjectId, ref: 'User' }],
    required: true,
    validate: [
      (arr: Types.ObjectId[]) => arr.length > 1 && arr.length <= 20,
      'Group DM must have between 2 and 20 members.'
    ]
  },
  avatarUrl: { type: String },
  lastMessageAt: { type: Date },
  createdAt: { type: Date, default: Date.now }
});

export default (mongoose.models.GroupDM as Model<IGroupDM>) || mongoose.model<IGroupDM>('GroupDM', GroupDMSchema); 