import mongoose, { Schema, Document, Model, Types } from 'mongoose';

export type ChannelType = 'text' | 'voice';

export interface IChannel extends Document {
  name: string;
  serverId: Types.ObjectId;
  type: ChannelType;
  createdAt: Date;
}

const ChannelSchema: Schema<IChannel> = new Schema({
  name: { type: String, required: true, trim: true },
  serverId: { type: Schema.Types.ObjectId, ref: 'Server', required: true },
  type: { type: String, enum: ['text', 'voice'], default: 'text' },
  createdAt: { type: Date, default: Date.now }
});

export default (mongoose.models.Channel as Model<IChannel>) || mongoose.model<IChannel>('Channel', ChannelSchema); 