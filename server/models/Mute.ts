import mongoose, { Schema, Document, Model, Types } from 'mongoose';

export interface IMute extends Document {
  serverId: Types.ObjectId;
  userId: Types.ObjectId;
  mutedBy: Types.ObjectId;
  reason?: string;
  createdAt: Date;
  expiresAt?: Date;
}

const MuteSchema: Schema<IMute> = new Schema({
  serverId: { type: Schema.Types.ObjectId, ref: 'Server', required: true },
  userId: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  mutedBy: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  reason: { type: String },
  createdAt: { type: Date, default: Date.now },
  expiresAt: { type: Date },
});

export default (mongoose.models.Mute as Model<IMute>) || mongoose.model<IMute>('Mute', MuteSchema); 