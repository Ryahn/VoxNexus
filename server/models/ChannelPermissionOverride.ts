import mongoose, { Schema, Document, Model, Types } from 'mongoose';

export interface IChannelPermissionOverride extends Document {
  channelId: Types.ObjectId;
  roleId?: Types.ObjectId;
  userId?: Types.ObjectId;
  allow: string[];
  deny: string[];
}

const ChannelPermissionOverrideSchema: Schema<IChannelPermissionOverride> = new Schema({
  channelId: { type: Schema.Types.ObjectId, ref: 'Channel', required: true },
  roleId: { type: Schema.Types.ObjectId, ref: 'Role' },
  userId: { type: Schema.Types.ObjectId, ref: 'User' },
  allow: { type: [String], default: [] },
  deny: { type: [String], default: [] },
});

export default (mongoose.models.ChannelPermissionOverride as Model<IChannelPermissionOverride>) || mongoose.model<IChannelPermissionOverride>('ChannelPermissionOverride', ChannelPermissionOverrideSchema); 